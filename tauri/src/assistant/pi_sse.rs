//! 健壮 SSE 解析器 —— 照搬 pi_agent_rust 的 `src/sse.rs`（行业标杆 agent 框架）。
//!
//! 相比 SuperTool 旧实现（`llm.rs` 里用 `from_utf8_lossy` + `buf.find('\n')`），
//! 这里解决了导致「流式响应中断」的三个根因：
//! 1. **UTF-8 跨 chunk 分片**：中文字符被 TCP 从中间切开时，旧实现两半各自变 U+FFFD，
//!    且 buffer 长度判断全错；这里用 `utf8_buffer` 把半个多字节序列留到下一轮再解。
//! 2. **只认 `\n` 不认 `\r`**：部分网关用裸 `\r` 或 `\r\n` 分行，旧实现会漏事件。
//! 3. **无空行结尾的挂尾事件**：流结束时旧实现直接丢弃没以空行结尾的最后一个事件；
//!    这里 `flush()` 会抢救它。

use std::borrow::Cow;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

const MAX_EVENT_DATA_BYTES: usize = 100 * 1024 * 1024;

/// 一个解析出的 SSE 事件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    /// 事件类型（来自 `event:` 字段，默认 "message"）。
    pub event: Cow<'static, str>,
    /// 事件数据（来自 `data:` 字段，多行用换行连接）。
    pub data: String,
    /// 最后事件 ID（来自 `id:` 字段）。
    pub id: Option<String>,
    /// 本事件是否显式含 `id:` 字段（SSE 规范会把 last id 带到后续事件）。
    pub id_was_explicit: bool,
    /// 重试间隔提示毫秒（来自 `retry:` 字段）。
    pub retry: Option<u64>,
}

impl Default for SseEvent {
    fn default() -> Self {
        Self {
            event: Cow::Borrowed("message"),
            data: String::new(),
            id: None,
            id_was_explicit: false,
            retry: None,
        }
    }
}

/// SSE 解析器状态。
#[derive(Debug)]
pub struct SseParser {
    buffer: String,
    current: SseEvent,
    has_data: bool,
    /// 是否已剥离首个 feed 的 BOM。
    bom_checked: bool,
    /// `buffer` 中已扫描过换行的字节数。
    scanned_len: usize,
    /// 单个事件的 data 累积上限（字节）。
    max_event_data_bytes: usize,
}

impl Default for SseParser {
    fn default() -> Self {
        Self {
            buffer: String::new(),
            current: SseEvent::default(),
            has_data: false,
            bom_checked: false,
            scanned_len: 0,
            max_event_data_bytes: MAX_EVENT_DATA_BYTES,
        }
    }
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn append_data_line(
        current: &mut SseEvent,
        value: &str,
        has_data: &mut bool,
        max_event_data_bytes: usize,
    ) {
        let projected_len = current
            .data
            .len()
            .saturating_add(value.len())
            .saturating_add(1);
        if projected_len > max_event_data_bytes {
            // 超限时仍标记 has_data，保事件边界，避免下游状态机等结束事件挂死
            *has_data = true;
            return;
        }
        current.data.push_str(value);
        current.data.push('\n');
        *has_data = true;
    }

    #[inline]
    fn parse_retry(value: &str) -> Option<u64> {
        if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) {
            value.parse().ok()
        } else {
            None
        }
    }

    /// 处理一行 SSE 数据。
    fn process_line(
        line: &str,
        current: &mut SseEvent,
        has_data: &mut bool,
        max_event_data_bytes: usize,
    ) {
        if let Some(rest) = line.strip_prefix(':') {
            // 注释行（可用于 keep-alive 心跳），忽略
            let _ = rest;
        } else if let Some((field, value)) = line.split_once(':') {
            let value = value.strip_prefix(' ').unwrap_or(value);
            match field {
                "event" => current.event = Cow::Owned(value.to_string()),
                "data" => Self::append_data_line(current, value, has_data, max_event_data_bytes),
                "id" if !value.contains('\0') => {
                    current.id = Some(value.to_string());
                    current.id_was_explicit = true;
                }
                "retry" => current.retry = Self::parse_retry(value),
                _ => {} // 未知字段忽略
            }
        } else {
            // 无值的字段
            match line {
                "event" => current.event = Cow::Borrowed(""),
                "data" => Self::append_data_line(current, "", has_data, max_event_data_bytes),
                "id" => {
                    current.id = Some(String::new());
                    current.id_was_explicit = true;
                }
                _ => {}
            }
        }
    }

    #[inline]
    fn reset_current_for_next_event(current: &mut SseEvent) {
        current.event = Cow::Borrowed("message");
        current.data.clear();
        current.id_was_explicit = false;
    }

    #[inline]
    fn carry_forward_event_state(current: &SseEvent) -> SseEvent {
        SseEvent {
            id: current.id.clone(),
            retry: current.retry,
            ..Default::default()
        }
    }

    /// 处理 `source` 中完整的行，通过 `emit` 派发事件。返回第一个未消费字节的偏移。
    #[inline]
    fn process_source<F>(
        source: &str,
        scan_start: usize,
        bom_checked: &mut bool,
        current: &mut SseEvent,
        has_data: &mut bool,
        max_event_data_bytes: usize,
        emit: &mut F,
    ) -> usize
    where
        F: FnMut(SseEvent),
    {
        let bytes = source.as_bytes();
        let mut start = 0usize;
        let mut search_pos = scan_start;

        // 剥离流开头的 UTF-8 BOM（SSE 规范要求）
        if !*bom_checked && !source.is_empty() {
            *bom_checked = true;
            if source.starts_with('\u{FEFF}') {
                start = 3;
                if search_pos < 3 {
                    search_pos = 3;
                }
            }
        }

        // 同时找 \r 和 \n（裸 CR / 裸 LF / CRLF 都支持）
        while search_pos < bytes.len() {
            let found = bytes[search_pos..]
                .iter()
                .position(|&b| b == b'\r' || b == b'\n');
            let Some(rel_pos) = found else { break };
            let pos = search_pos + rel_pos;
            let b = bytes[pos];

            let line_end;
            let next_start;

            if b == b'\n' {
                // 裸 LF
                line_end = pos;
                next_start = pos + 1;
            } else {
                // 找到 \r
                if pos + 1 < source.len() {
                    line_end = pos;
                    next_start = if bytes[pos + 1] == b'\n' {
                        // CRLF
                        pos + 2
                    } else {
                        // 裸 CR
                        pos + 1
                    };
                } else {
                    // 缓冲区末尾的 CR —— 等更多数据判断是否为 CRLF
                    break;
                }
            }

            let line = &source[start..line_end];
            start = next_start;
            search_pos = next_start;

            if line.is_empty() {
                // 空行 = 事件边界
                if *has_data {
                    // 去掉 data 末尾多余换行
                    if current.data.ends_with('\n') {
                        current.data.pop();
                    }
                    // 空事件名按 "message" 派发
                    if current.event.is_empty() {
                        current.event = Cow::Borrowed("message");
                    }
                    let next_event = Self::carry_forward_event_state(current);
                    emit(std::mem::take(current));
                    *current = next_event;
                    *has_data = false;
                } else {
                    Self::reset_current_for_next_event(current);
                }
            } else {
                Self::process_line(line, current, has_data, max_event_data_bytes);
            }
        }

        start
    }

    /// 喂数据并派发完整事件。
    fn feed_into<F>(&mut self, data: &str, mut emit: F)
    where
        F: FnMut(SseEvent),
    {
        const MAX_BUFFER_SIZE: usize = 10 * 1024 * 1024;
        if self.buffer.is_empty() {
            // 快路径：直接处理，不复制到 buffer
            let consumed = Self::process_source(
                data,
                0,
                &mut self.bom_checked,
                &mut self.current,
                &mut self.has_data,
                self.max_event_data_bytes,
                &mut emit,
            );
            if consumed < data.len() {
                self.buffer.push_str(&data[consumed..]);
            }
        } else {
            // 慢路径：拼到临时源上，只保留真正未消费的尾部
            let mut combined = std::mem::take(&mut self.buffer);
            combined.push_str(data);
            // 从上次安全点重新扫（减 1 处理跨 chunk 的 CRLF）
            let scan_start = self.scanned_len.saturating_sub(1);
            let consumed = Self::process_source(
                &combined,
                scan_start,
                &mut self.bom_checked,
                &mut self.current,
                &mut self.has_data,
                self.max_event_data_bytes,
                &mut emit,
            );
            if consumed < combined.len() {
                self.buffer = combined[consumed..].to_string();
            } else {
                self.buffer.clear();
            }
        }
        if self.buffer.len() > MAX_BUFFER_SIZE {
            self.buffer = String::new();
            self.current = SseEvent::default();
            self.has_data = false;
            self.bom_checked = false;
            self.scanned_len = 0;
            emit(SseEvent {
                event: Cow::Borrowed("error"),
                data: "SSE buffer limit exceeded".to_string(),
                ..Default::default()
            });
            return;
        }
        // 无论是否耗尽，剩余 buffer 都已扫描完
        self.scanned_len = self.buffer.len();
    }

    /// 喂数据，返回解析出的完整事件。
    pub fn feed(&mut self, data: &str) -> Vec<SseEvent> {
        let mut events = Vec::with_capacity(4);
        self.feed_into(data, |event| events.push(event));
        events
    }

    /// 是否有待处理数据。
    pub const fn has_pending(&self) -> bool {
        !self.buffer.is_empty() || self.has_data
    }

    /// 冲刷待处理事件（流结束时调用，抢救无空行结尾的挂尾事件）。
    pub fn flush(&mut self) -> Option<SseEvent> {
        if !self.buffer.is_empty() {
            let line = std::mem::take(&mut self.buffer);
            let line = line.trim_end_matches('\r');
            Self::process_line(
                line,
                &mut self.current,
                &mut self.has_data,
                self.max_event_data_bytes,
            );
        }
        if self.has_data {
            if self.current.data.ends_with('\n') {
                self.current.data.pop();
            }
            if self.current.event.is_empty() {
                self.current.event = Cow::Borrowed("message");
            }
            let event = std::mem::take(&mut self.current);
            self.current = SseEvent::default();
            self.has_data = false;
            Some(event)
        } else {
            None
        }
    }
}

/// 把字节流包装成 SSE 事件流（处理 UTF-8 跨 chunk 分片）。
pub struct SseStream<S> {
    inner: S,
    parser: SseParser,
    pending_events: VecDeque<SseEvent>,
    pending_error: Option<std::io::Error>,
    pending_error_is_terminal: bool,
    terminated: bool,
    utf8_buffer: Vec<u8>,
}

impl<S> SseStream<S> {
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            parser: SseParser::new(),
            pending_events: VecDeque::new(),
            pending_error: None,
            pending_error_is_terminal: false,
            terminated: false,
            utf8_buffer: Vec::new(),
        }
    }
}

impl<S> SseStream<S>
where
    S: futures::Stream<Item = Result<Vec<u8>, std::io::Error>> + Unpin,
{
    #[inline]
    fn invalid_utf8_error() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8 in SSE stream")
    }

    #[inline]
    fn feed_parsed_chunk(
        parser: &mut SseParser,
        pending: &mut VecDeque<SseEvent>,
        s: &str,
    ) {
        parser.feed_into(s, |event| pending.push_back(event));
    }

    fn process_chunk_without_utf8_tail(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        let mut processed = 0;
        let mut first_error: Option<std::io::Error> = None;
        loop {
            match std::str::from_utf8(&bytes[processed..]) {
                Ok(s) => {
                    if !s.is_empty() {
                        Self::feed_parsed_chunk(&mut self.parser, &mut self.pending_events, s);
                    }
                    return first_error.map_or(Ok(()), Err);
                }
                Err(err) => {
                    let valid_len = err.valid_up_to();
                    if valid_len > 0 {
                        let s = std::str::from_utf8(&bytes[processed..processed + valid_len])
                            .map_err(std::io::Error::other)?;
                        Self::feed_parsed_chunk(&mut self.parser, &mut self.pending_events, s);
                        processed += valid_len;
                    }
                    if let Some(invalid_len) = err.error_len() {
                        processed += invalid_len;
                        if first_error.is_none() {
                            first_error = Some(Self::invalid_utf8_error());
                        }
                    } else {
                        // 末尾是不完整 UTF-8 序列，留到下一轮
                        self.utf8_buffer.extend_from_slice(&bytes[processed..]);
                        return first_error.map_or(Ok(()), Err);
                    }
                }
            }
        }
    }

    fn process_chunk_with_utf8_tail(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.utf8_buffer.extend_from_slice(bytes);
        let mut processed = 0;
        let mut first_error: Option<std::io::Error> = None;
        loop {
            match std::str::from_utf8(&self.utf8_buffer[processed..]) {
                Ok(s) => {
                    if !s.is_empty() {
                        Self::feed_parsed_chunk(&mut self.parser, &mut self.pending_events, s);
                    }
                    self.utf8_buffer.clear();
                    return first_error.map_or(Ok(()), Err);
                }
                Err(err) => {
                    let valid_len = err.valid_up_to();
                    if valid_len > 0 {
                        let s = std::str::from_utf8(
                            &self.utf8_buffer[processed..processed + valid_len],
                        )
                        .map_err(std::io::Error::other)?;
                        Self::feed_parsed_chunk(&mut self.parser, &mut self.pending_events, s);
                        processed += valid_len;
                    }
                    if let Some(invalid_len) = err.error_len() {
                        processed += invalid_len;
                        if first_error.is_none() {
                            first_error = Some(Self::invalid_utf8_error());
                        }
                    } else {
                        // 移未消费的尾部字节到 buffer 开头
                        let remaining = self.utf8_buffer.len() - processed;
                        self.utf8_buffer.copy_within(processed.., 0);
                        self.utf8_buffer.truncate(remaining);
                        return first_error.map_or(Ok(()), Err);
                    }
                }
            }
        }
    }

    fn process_chunk(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        if self.utf8_buffer.is_empty() {
            self.process_chunk_without_utf8_tail(bytes)
        } else {
            self.process_chunk_with_utf8_tail(bytes)
        }
    }

    fn poll_stream_end(&mut self) -> Poll<Option<Result<SseEvent, std::io::Error>>> {
        if !self.utf8_buffer.is_empty() {
            // EOF 时残留不完整 UTF-8 序列 = 终结性错误
            self.utf8_buffer.clear();
            self.pending_events.clear();
            self.pending_error = None;
            self.parser = SseParser::new();
            self.terminated = true;
            return Poll::Ready(Some(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Stream ended with incomplete UTF-8 sequence",
            ))));
        }
        if let Some(event) = self.parser.flush() {
            self.terminated = true;
            return Poll::Ready(Some(Ok(event)));
        }
        self.terminated = true;
        Poll::Ready(None)
    }

    /// 取下一个 SSE 事件。
    pub fn poll_next_event(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<SseEvent, std::io::Error>>> {
        if let Some(event) = self.pending_events.pop_front() {
            return Poll::Ready(Some(Ok(event)));
        }
        if let Some(err) = self.pending_error.take() {
            if self.pending_error_is_terminal {
                self.pending_error_is_terminal = false;
                self.pending_events.clear();
                self.utf8_buffer.clear();
                self.parser = SseParser::new();
                self.terminated = true;
            }
            return Poll::Ready(Some(Err(err)));
        }
        if self.terminated {
            return Poll::Ready(None);
        }

        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    if let Err(err) = self.process_chunk(&bytes) {
                        if let Some(event) = self.pending_events.pop_front() {
                            self.pending_error = Some(err);
                            self.pending_error_is_terminal = true;
                            return Poll::Ready(Some(Ok(event)));
                        }
                        self.pending_events.clear();
                        self.utf8_buffer.clear();
                        self.parser = SseParser::new();
                        self.terminated = true;
                        return Poll::Ready(Some(Err(err)));
                    }
                    if let Some(event) = self.pending_events.pop_front() {
                        return Poll::Ready(Some(Ok(event)));
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Ready(None) => {
                    return self.poll_stream_end();
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

impl<S> futures::Stream for SseStream<S>
where
    S: futures::Stream<Item = Result<Vec<u8>, std::io::Error>> + Unpin,
{
    type Item = Result<SseEvent, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.poll_next_event(cx)
    }
}

// =================== 流式 JSON 补全（照搬 pi openai.rs） ===================
//
// 流式 tool_call 参数是分片增量到达的，累积串经常是「半截 JSON」。
// 若直接解析失败就丢弃整个工具调用，会丢掉该工具调用导致回复中断。
// 这里对半截 JSON 做「补引号 + 补右括号 + 裁剪悬空 key/逗号」尽力补全。

/// 尽力把流式累积的 JSON 前缀补全成合法 JSON。
/// 快速路径：本身已是合法 JSON 直接返回。否则做结构扫描补全。
pub fn complete_partial_json(input: &str) -> Option<serde_json::Value> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(s) {
        return Some(value);
    }

    // 结构扫描，学习还有哪些括号没闭合
    let mut closers: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    for byte in s.bytes() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => closers.push('}'),
            b'[' => closers.push(']'),
            b'}' | b']' => {
                closers.pop();
            }
            _ => {}
        }
    }

    let mut out = String::from(s);
    if in_string {
        if escaped {
            // 悬空的转义反斜杠（`..."ab\`），补引号前先丢掉
            out.pop();
        }
        out.push('"');
    }

    // 逐层补右括号，闭合前裁剪悬空尾部
    while let Some(closer) = closers.pop() {
        trim_dangling_json_tail(&mut out, closer == '}');
        out.push(closer);
    }

    serde_json::from_str::<serde_json::Value>(&out).ok()
}

/// 追加容器右括号前，丢弃悬空逗号，以及（对象）悬空的 `"key":` 或裸 `"key"`。
fn trim_dangling_json_tail(out: &mut String, is_object: bool) {
    loop {
        let before = out.len();
        while out.ends_with(char::is_whitespace) {
            out.pop();
        }
        if out.ends_with(',') {
            out.pop();
            continue;
        }
        if is_object {
            // `"key":` 无值 → 丢掉冒号和 key 字符串
            if out.ends_with(':') {
                out.pop();
                while out.ends_with(char::is_whitespace) {
                    out.pop();
                }
                remove_trailing_json_string(out);
                continue;
            }
            // 尾部字符串：若前面是 `:` 则是完整 value（结束并闭合）；
            // 否则（前面是 `,` 或 `{`）是悬空 key → 丢掉
            if let Some(start) = trailing_json_string_start(out) {
                let preceded_by_colon = out[..start].trim_end().ends_with(':');
                if preceded_by_colon {
                    break;
                }
                out.truncate(start);
                continue;
            }
        }
        if out.len() == before {
            break;
        }
    }
}

/// `out` 末尾以 JSON 字符串字面量结尾时，返回其开引号的字节索引。
fn trailing_json_string_start(out: &str) -> Option<usize> {
    let bytes = out.as_bytes();
    if bytes.last() != Some(&b'"') {
        return None;
    }
    let mut i = bytes.len() - 1;
    while i > 0 {
        i -= 1;
        if bytes[i] == b'"' {
            // 数前面反斜杠判断是转义引号还是开引号
            let mut backslashes = 0usize;
            let mut j = i;
            while j > 0 && bytes[j - 1] == b'\\' {
                backslashes += 1;
                j -= 1;
            }
            if backslashes.is_multiple_of(2) {
                return Some(i);
            }
        }
    }
    None
}

/// 从 `out` 移除末尾完整的 JSON 字符串字面量。
fn remove_trailing_json_string(out: &mut String) {
    if let Some(start) = trailing_json_string_start(out) {
        out.truncate(start);
    }
}

// =================== 测试 ===================

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use futures::stream;

    fn parse_all(input: &str) -> Vec<SseEvent> {
        let mut parser = SseParser::new();
        let mut events = parser.feed(input);
        if let Some(event) = parser.flush() {
            events.push(event);
        }
        events
    }

    #[test]
    fn test_simple_event() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: hello\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn test_multiline_data() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: line1\ndata: line2\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "line1\nline2");
    }

    #[test]
    fn test_incremental_feed() {
        let mut parser = SseParser::new();
        assert!(parser.feed("data: hel").is_empty());
        assert!(parser.feed("lo\n").is_empty());
        let events = parser.feed("\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn test_crlf_handling() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: hello\r\n\r\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello");
    }

    #[test]
    fn test_bare_cr_line_ending() {
        let mut parser = SseParser::new();
        let events = parser.feed("data: hello\r\ndata: world\r\n\r\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "hello\nworld");
    }

    #[test]
    fn test_flush_pending_without_trailing_blank_line() {
        // 流结束时没有空行结尾的挂尾事件必须被 flush 抢救
        let mut parser = SseParser::new();
        assert!(parser.feed("data: incomplete").is_empty());
        let event = parser.flush();
        assert!(event.is_some());
        assert_eq!(event.unwrap().data, "incomplete");
    }

    #[test]
    fn test_sse_stream_utf8_split_across_chunks() {
        let chunks = vec![
            Ok(b"data: \xE6\x95".to_vec()),
            Ok(b"\xB0\xE6\x8D\xAE\n\n".to_vec()),
        ];
        let mut stream = SseStream::new(stream::iter(chunks));
        futures::executor::block_on(async {
            let event = stream.next().await.expect("event").expect("ok");
            assert_eq!(event.data, "数据");
            assert!(stream.next().await.is_none());
        });
    }

    #[test]
    fn test_comment_ignored() {
        let mut parser = SseParser::new();
        let events = parser.feed(": keepalive\ndata: actual\n\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "actual");
    }

    #[test]
    fn test_rapid_sequential_events() {
        let mut parser = SseParser::new();
        let mut input = String::new();
        for i in 0..200 {
            input.push_str(&format!("data: payload{i}\n\n"));
        }
        let events = parser.feed(&input);
        assert_eq!(events.len(), 200);
        assert_eq!(events[199].data, "payload199");
    }

    // ========== complete_partial_json ==========

    #[test]
    fn partial_json_completes_object() {
        // 流式 tool_call 参数半截：`{"key":"va`
        let v = complete_partial_json(r#"{"key":"va"#).unwrap();
        assert_eq!(v, serde_json::json!({"key": "va"}));
    }

    #[test]
    fn partial_json_completes_with_dangling_comma() {
        let v = complete_partial_json(r#"{"a":1,"b":2,"#).unwrap();
        assert_eq!(v, serde_json::json!({"a": 1, "b": 2}));
    }

    #[test]
    fn partial_json_completes_nested_and_dangling_key() {
        let v = complete_partial_json(r#"{"outer":{"inner":[1,2],"key"#).unwrap();
        assert_eq!(v, serde_json::json!({"outer": {"inner": [1, 2]}}));
    }

    #[test]
    fn partial_json_handles_escaped_quotes() {
        let v = complete_partial_json(r#"{"path":"C:\\Users\\nam"#).unwrap();
        assert_eq!(v, serde_json::json!({"path": "C:\\Users\\nam"}));
    }

    #[test]
    fn partial_json_returns_none_on_garbage() {
        assert!(complete_partial_json("").is_none());
        assert!(complete_partial_json("   ").is_none());
        assert!(complete_partial_json("not json at all").is_none());
    }

    #[test]
    fn partial_json_valid_input_passthrough() {
        let v = complete_partial_json(r#"{"a":1}"#).unwrap();
        assert_eq!(v, serde_json::json!({"a": 1}));
    }
}
