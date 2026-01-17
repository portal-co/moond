/**
 * AGC WebSocket Client
 * 
 * Provides real-time data synchronization for Apollo Guidance Computer UI components.
 * Uses JSON-patch (RFC 6902) style diffs for minimal network overhead.
 * 
 * Protocol:
 * ---------
 * 1. On connect: Server sends full state snapshot:
 *    { "type": "snapshot", "state": { ... } }
 * 
 * 2. On update: Server sends diff patches:
 *    { "type": "patch", "ops": [
 *        { "op": "replace", "path": "/registers/r1", "value": "+12345" },
 *        { "op": "replace", "path": "/status/UPLINK", "value": true }
 *    ]}
 * 
 * 3. Client can send commands:
 *    { "type": "keypress", "key": "VERB" }
 *    { "type": "input", "value": "06" }
 *    { "type": "bit_toggle", "row": "Address", "bit": 5 }
 * 
 * State Schema:
 * -------------
 * {
 *   "registers": { "prog": "66", "verb": "06", "noun": "60", "r1": "+00000", "r2": "+00000", "r3": "+00000" },
 *   "status": { "UPLINK": false, "TEMP": false, "NO ATT": false, ... },
 *   "bits": { "Address": [0,0,1,0,...], "Data HI": [...], ... }
 * }
 */

export class AGCClient extends EventTarget {
    /** @type {WebSocket|null} */
    #ws = null;
    
    /** @type {Object} Current synchronized state */
    #state = {};
    
    /** @type {string} */
    #url;
    
    /** @type {number} Reconnect delay in ms */
    #reconnectDelay = 1000;
    
    /** @type {number|null} */
    #reconnectTimer = null;
    
    /** @type {boolean} */
    #autoReconnect = true;

    /**
     * @param {string} [url] - WebSocket URL, defaults to ws://localhost:8080/agc
     */
    constructor(url = `ws://${location.host}/agc`) {
        super();
        this.#url = url;
    }

    /** Current state (read-only copy) */
    get state() {
        return structuredClone(this.#state);
    }

    /** Connection status */
    get connected() {
        return this.#ws?.readyState === WebSocket.OPEN;
    }

    /**
     * Connect to the AGC WebSocket server
     * @returns {Promise<void>}
     */
    connect() {
        return new Promise((resolve, reject) => {
            if (this.#ws) {
                this.#ws.close();
            }

            this.#ws = new WebSocket(this.#url);

            this.#ws.onopen = () => {
                this.#reconnectDelay = 1000;
                this.dispatchEvent(new CustomEvent('connected'));
                resolve();
            };

            this.#ws.onerror = (e) => {
                this.dispatchEvent(new CustomEvent('error', { detail: e }));
                reject(e);
            };

            this.#ws.onclose = () => {
                this.dispatchEvent(new CustomEvent('disconnected'));
                if (this.#autoReconnect) {
                    this.#scheduleReconnect();
                }
            };

            this.#ws.onmessage = (e) => this.#handleMessage(e.data);
        });
    }

    /**
     * Disconnect from the server
     */
    disconnect() {
        this.#autoReconnect = false;
        if (this.#reconnectTimer) {
            clearTimeout(this.#reconnectTimer);
        }
        this.#ws?.close();
        this.#ws = null;
    }

    #scheduleReconnect() {
        this.#reconnectTimer = setTimeout(() => {
            this.#reconnectDelay = Math.min(this.#reconnectDelay * 2, 30000);
            this.connect().catch(() => {});
        }, this.#reconnectDelay);
    }

    /**
     * Handle incoming WebSocket messages
     * @param {string} data 
     */
    #handleMessage(data) {
        try {
            const msg = JSON.parse(data);
            
            switch (msg.type) {
                case 'snapshot':
                    this.#state = msg.state;
                    this.dispatchEvent(new CustomEvent('snapshot', { detail: this.#state }));
                    break;
                    
                case 'patch':
                    this.#applyPatches(msg.ops);
                    this.dispatchEvent(new CustomEvent('patch', { detail: msg.ops }));
                    break;
                    
                default:
                    console.warn('Unknown message type:', msg.type);
            }
            
            this.dispatchEvent(new CustomEvent('statechange', { detail: this.#state }));
        } catch (e) {
            console.error('Failed to parse message:', e);
        }
    }

    /**
     * Apply JSON-patch style operations to state
     * @param {Array<{op: string, path: string, value: any}>} ops 
     */
    #applyPatches(ops) {
        for (const op of ops) {
            const pathParts = op.path.split('/').filter(Boolean);
            
            switch (op.op) {
                case 'replace':
                case 'add':
                    this.#setPath(pathParts, op.value);
                    break;
                case 'remove':
                    this.#deletePath(pathParts);
                    break;
            }
        }
    }

    #setPath(parts, value) {
        let obj = this.#state;
        for (let i = 0; i < parts.length - 1; i++) {
            if (!(parts[i] in obj)) obj[parts[i]] = {};
            obj = obj[parts[i]];
        }
        obj[parts[parts.length - 1]] = value;
    }

    #deletePath(parts) {
        let obj = this.#state;
        for (let i = 0; i < parts.length - 1; i++) {
            if (!(parts[i] in obj)) return;
            obj = obj[parts[i]];
        }
        delete obj[parts[parts.length - 1]];
    }

    /**
     * Send a keypress command to the AGC
     * @param {string} key - Key identifier (e.g., 'VERB', 'NOUN', '0'-'9', '+', '-')
     */
    sendKeypress(key) {
        this.#send({ type: 'keypress', key });
    }

    /**
     * Send numeric input
     * @param {string} value 
     */
    sendInput(value) {
        this.#send({ type: 'input', value });
    }

    /**
     * Toggle a bit in the binary control panel
     * @param {string} row - Row label (e.g., 'Address', 'Data HI')
     * @param {number} bit - Bit index (0-15)
     */
    sendBitToggle(row, bit) {
        this.#send({ type: 'bit_toggle', row, bit });
    }

    /**
     * Send raw command to server
     * @param {Object} cmd 
     */
    #send(cmd) {
        if (this.connected) {
            this.#ws.send(JSON.stringify(cmd));
        }
    }
}

/**
 * Utility: Get value at JSON pointer path from state
 * @param {Object} state 
 * @param {string} path - JSON pointer (e.g., '/registers/r1')
 * @returns {any}
 */
export function getPath(state, path) {
    const parts = path.split('/').filter(Boolean);
    let obj = state;
    for (const p of parts) {
        if (obj == null) return undefined;
        obj = obj[p];
    }
    return obj;
}

/**
 * Default client instance for convenience
 */
export const agcClient = new AGCClient();
