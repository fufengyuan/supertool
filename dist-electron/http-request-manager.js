"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.registerHttpApiHandlers = registerHttpApiHandlers;
const electron_1 = require("electron");
const db = require("./database");
function executeHttpRequest(config) {
    const startTime = Date.now();
    const { method = 'GET', url, headers = {}, body, contentType, timeout = 30000, } = config;
    try {
        if (!url)
            throw new Error('URL is required');
        const urlObj = new URL(url);
        const httpModule = urlObj.protocol === 'https:' ? require('https') : require('http');
        // Resolve content type
        const contentTypeMap = {
            json: 'application/json',
            xml: 'application/xml',
            'form-data': 'multipart/form-data',
            'x-www-form-urlencoded': 'application/x-www-form-urlencoded',
            text: 'text/plain',
        };
        let resolvedContentType = contentTypeMap[contentType || ''] || contentType || '';
        if (!resolvedContentType && body !== undefined && body !== null) {
            if (typeof body === 'object')
                resolvedContentType = 'application/json';
        }
        // Build request headers
        const reqHeaders = { ...headers };
        if (resolvedContentType)
            reqHeaders['Content-Type'] = resolvedContentType;
        // Serialize body
        let requestBody;
        if (body !== undefined && body !== null) {
            if (typeof body === 'object')
                requestBody = JSON.stringify(body);
            else if (typeof body === 'string')
                requestBody = body;
            else
                requestBody = String(body);
            reqHeaders['Content-Length'] = Buffer.byteLength(requestBody).toString();
        }
        const requestOptions = {
            method: method.toUpperCase(),
            hostname: urlObj.hostname,
            port: urlObj.port || (urlObj.protocol === 'https:' ? 443 : 80),
            path: urlObj.pathname + urlObj.search,
            headers: reqHeaders,
            timeout,
            rejectUnauthorized: false,
        };
        return new Promise((resolve) => {
            const req = httpModule.request(requestOptions, (res) => {
                const chunks = [];
                res.on('data', (chunk) => chunks.push(chunk));
                res.on('end', () => {
                    const elapsed = Date.now() - startTime;
                    const responseBuffer = Buffer.concat(chunks);
                    const responseBody = responseBuffer.toString('utf-8');
                    let parsedBody = responseBody;
                    const resContentType = (res.headers['content-type'] || '').toLowerCase();
                    if (resContentType.includes('application/json')) {
                        try {
                            parsedBody = JSON.parse(responseBody);
                        }
                        catch {
                            parsedBody = responseBody;
                        }
                    }
                    const responseHeaders = {};
                    for (const [key, value] of Object.entries(res.headers)) {
                        responseHeaders[key] = Array.isArray(value) ? value.join(', ') : String(value);
                    }
                    resolve({
                        status: res.statusCode, statusText: res.statusMessage,
                        headers: responseHeaders, body: parsedBody, rawBody: responseBody,
                        time: elapsed, size: responseBuffer.length, error: null,
                    });
                });
            });
            req.on('timeout', () => {
                req.destroy();
                resolve({ status: 0, statusText: 'Timeout', headers: {}, body: null, rawBody: null, time: Date.now() - startTime, size: 0, error: `Request timed out after ${timeout}ms` });
            });
            req.on('error', (err) => {
                resolve({ status: 0, statusText: 'Error', headers: {}, body: null, rawBody: null, time: Date.now() - startTime, size: 0, error: err.message });
            });
            if (requestBody !== undefined)
                req.write(requestBody);
            req.end();
        });
    }
    catch (err) {
        return Promise.resolve({
            status: 0, statusText: 'Error', headers: {}, body: null, rawBody: null,
            time: Date.now() - startTime, size: 0, error: err.message || 'Unknown error',
        });
    }
}
function registerHttpApiHandlers() {
    electron_1.ipcMain.handle('api:http-request', async (_event, config) => executeHttpRequest(config));
    electron_1.ipcMain.handle('api:requests:get-all', () => {
        try {
            return { success: true, rows: db.getApiRequests() };
        }
        catch (e) {
            return { success: false, error: e.message };
        }
    });
    electron_1.ipcMain.handle('api:requests:add', (_event, req) => {
        try {
            const request = db.addApiRequest(req);
            return { success: true, id: request.id };
        }
        catch (e) {
            return { success: false, error: e.message };
        }
    });
    electron_1.ipcMain.handle('api:requests:update', (_event, id, updates) => {
        try {
            db.updateApiRequest(id, updates);
            return { success: true };
        }
        catch (e) {
            return { success: false, error: e.message };
        }
    });
    electron_1.ipcMain.handle('api:requests:delete', (_event, id) => {
        try {
            db.deleteApiRequest(id);
            return { success: true };
        }
        catch (e) {
            return { success: false, error: e.message };
        }
    });
}
//# sourceMappingURL=http-request-manager.js.map