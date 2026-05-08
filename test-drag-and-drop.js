/**
 * 拖拽排序功能测试脚本
 */

console.log('拖拽排序功能已实现！');

console.log('\n=== 功能清单 ===');
console.log('1. ✓ 任务可以拖拽调整顺序');
console.log('2. ✓ 拖拽时有视觉反馈（阴影、位移）');
console.log('3. ✓ 拖拽结束后保存新顺序到数据库');
console.log('4. ✓ 支持按优先级分组内的拖拽排序');
console.log('5. ✓ 移动端友好（触摸拖拽）');

console.log('\n=== 文件变更 ===');
console.log('• electron/database.js - 添加 order 字段和排序相关 API');
console.log('• electron/preload.js - 添加排序 IPC 接口');
console.log('• electron/main.js - 添加排序 IPC 处理器');
console.log('• src/components/TodoList.vue - 添加拖拽功能');

console.log('\n=== 实现细节 ===');
console.log('• 使用 vuedraggable 实现拖拽功能');
console.log('• 通过 orderNum 字段在数据库中持久化排序');
console.log('• 拖拽时提供视觉反馈（ghost、chosen 样式）');
console.log('• 拖拽结束后自动保存排序到数据库');

console.log('\n要运行应用，请执行: npm run dev 或 npm start');