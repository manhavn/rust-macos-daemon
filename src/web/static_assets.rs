pub const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>macOS Launchd Daemon Manager</title>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg-dark: #0b0f19;
            --card-bg: rgba(22, 30, 49, 0.7);
            --card-border: rgba(255, 255, 255, 0.08);
            --accent-cyan: #00f2fe;
            --accent-blue: #4facfe;
            --accent-emerald: #10b981;
            --accent-amber: #f59e0b;
            --accent-rose: #f43f5e;
            --accent-purple: #a855f7;
            --text-primary: #f8fafc;
            --text-secondary: #94a3b8;
            --text-muted: #64748b;
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        body {
            font-family: 'Plus Jakarta Sans', -apple-system, BlinkMacSystemFont, sans-serif;
            background-color: var(--bg-dark);
            color: var(--text-primary);
            background-image: 
                radial-gradient(at 0% 0%, rgba(79, 172, 254, 0.12) 0px, transparent 50%),
                radial-gradient(at 100% 0%, rgba(168, 85, 247, 0.12) 0px, transparent 50%),
                radial-gradient(at 50% 100%, rgba(16, 185, 129, 0.08) 0px, transparent 50%);
            background-attachment: fixed;
            min-height: 100vh;
            line-height: 1.5;
        }

        .navbar {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 1.25rem 2rem;
            background: rgba(11, 15, 25, 0.8);
            backdrop-filter: blur(16px);
            border-bottom: 1px solid var(--card-border);
            position: sticky;
            top: 0;
            z-index: 100;
        }

        .brand {
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }

        .brand-icon {
            width: 38px;
            height: 38px;
            border-radius: 10px;
            background: linear-gradient(135deg, var(--accent-cyan), var(--accent-blue));
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: 800;
            color: #0b0f19;
            font-size: 1.2rem;
            box-shadow: 0 0 15px rgba(0, 242, 254, 0.3);
        }

        .brand-title h1 {
            font-size: 1.25rem;
            font-weight: 700;
            background: linear-gradient(135deg, #ffffff, #cbd5e1);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }

        .brand-title p {
            font-size: 0.75rem;
            color: var(--text-secondary);
        }

        .sys-status {
            display: flex;
            align-items: center;
            gap: 1rem;
        }

        .badge {
            display: inline-flex;
            align-items: center;
            gap: 0.4rem;
            padding: 0.35rem 0.85rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 600;
            letter-spacing: 0.02em;
        }

        .badge-root {
            background: rgba(244, 63, 94, 0.15);
            color: #fda4af;
            border: 1px solid rgba(244, 63, 94, 0.3);
        }

        .badge-user {
            background: rgba(16, 185, 129, 0.15);
            color: #6ee7b7;
            border: 1px solid rgba(16, 185, 129, 0.3);
        }

        .badge-pulse {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background-color: currentColor;
            animation: pulse 2s infinite;
        }

        @keyframes pulse {
            0% { opacity: 1; transform: scale(1); }
            50% { opacity: 0.4; transform: scale(1.2); }
            100% { opacity: 1; transform: scale(1); }
        }

        .btn {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            gap: 0.5rem;
            padding: 0.55rem 1.2rem;
            border-radius: 8px;
            font-size: 0.85rem;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s ease;
            border: 1px solid transparent;
            font-family: inherit;
        }

        .btn-primary {
            background: linear-gradient(135deg, var(--accent-cyan), var(--accent-blue));
            color: #0b0f19;
            box-shadow: 0 4px 12px rgba(0, 242, 254, 0.25);
        }

        .btn-primary:hover {
            opacity: 0.9;
            transform: translateY(-1px);
            box-shadow: 0 6px 16px rgba(0, 242, 254, 0.35);
        }

        .btn-secondary {
            background: rgba(255, 255, 255, 0.05);
            color: var(--text-primary);
            border-color: var(--card-border);
        }

        .btn-secondary:hover {
            background: rgba(255, 255, 255, 0.1);
        }

        .btn-danger {
            background: rgba(244, 63, 94, 0.15);
            color: #fda4af;
            border-color: rgba(244, 63, 94, 0.3);
        }

        .btn-danger:hover {
            background: rgba(244, 63, 94, 0.3);
        }

        .btn-sm {
            padding: 0.35rem 0.7rem;
            font-size: 0.75rem;
            border-radius: 6px;
        }

        .container {
            max-width: 1300px;
            margin: 2rem auto;
            padding: 0 1.5rem;
        }

        .kpi-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
            gap: 1.25rem;
            margin-bottom: 2rem;
        }

        .kpi-card {
            background: var(--card-bg);
            backdrop-filter: blur(12px);
            border: 1px solid var(--card-border);
            border-radius: 14px;
            padding: 1.25rem;
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }

        .kpi-title {
            font-size: 0.8rem;
            font-weight: 600;
            color: var(--text-secondary);
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }

        .kpi-value {
            font-size: 1.8rem;
            font-weight: 700;
            color: var(--text-primary);
        }

        .toolbar {
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-wrap: wrap;
            gap: 1rem;
            margin-bottom: 1.5rem;
            background: var(--card-bg);
            padding: 1rem 1.25rem;
            border-radius: 12px;
            border: 1px solid var(--card-border);
        }

        .tabs {
            display: flex;
            gap: 0.5rem;
        }

        .tab {
            padding: 0.45rem 1rem;
            border-radius: 8px;
            font-size: 0.825rem;
            font-weight: 600;
            color: var(--text-secondary);
            cursor: pointer;
            transition: all 0.2s;
            background: transparent;
            border: none;
        }

        .tab.active {
            background: rgba(255, 255, 255, 0.1);
            color: var(--accent-cyan);
            border: 1px solid rgba(0, 242, 254, 0.2);
        }

        .search-box {
            display: flex;
            align-items: center;
            background: rgba(11, 15, 25, 0.6);
            border: 1px solid var(--card-border);
            border-radius: 8px;
            padding: 0.45rem 0.85rem;
            width: 280px;
        }

        .search-box input {
            background: transparent;
            border: none;
            outline: none;
            color: var(--text-primary);
            font-family: inherit;
            font-size: 0.85rem;
            width: 100%;
        }

        .service-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
            gap: 1.25rem;
        }

        .service-card {
            background: var(--card-bg);
            backdrop-filter: blur(12px);
            border: 1px solid var(--card-border);
            border-radius: 14px;
            padding: 1.25rem;
            display: flex;
            flex-direction: column;
            justify-content: space-between;
            transition: transform 0.2s, box-shadow 0.2s;
        }

        .service-card:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
            border-color: rgba(255, 255, 255, 0.15);
        }

        .service-header {
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 0.75rem;
        }

        .service-title {
            font-size: 0.95rem;
            font-weight: 700;
            color: var(--text-primary);
            word-break: break-all;
            margin-bottom: 0.25rem;
        }

        .scope-tag {
            font-size: 0.7rem;
            padding: 0.2rem 0.5rem;
            border-radius: 4px;
            font-weight: 600;
            text-transform: uppercase;
        }

        .scope-user { background: rgba(16, 185, 129, 0.15); color: #34d399; }
        .scope-global { background: rgba(59, 130, 246, 0.15); color: #60a5fa; }
        .scope-system { background: rgba(168, 85, 247, 0.15); color: #c084fc; }

        .service-meta {
            font-size: 0.775rem;
            color: var(--text-secondary);
            display: flex;
            flex-direction: column;
            gap: 0.35rem;
            margin-bottom: 1.25rem;
            background: rgba(11, 15, 25, 0.4);
            padding: 0.75rem;
            border-radius: 8px;
            font-family: 'JetBrains Mono', monospace;
        }

        .meta-line {
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }

        .service-actions {
            display: flex;
            gap: 0.4rem;
            flex-wrap: wrap;
        }

        /* Modal styling */
        .modal-overlay {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.75);
            backdrop-filter: blur(8px);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 1000;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.25s ease;
        }

        .modal-overlay.active {
            opacity: 1;
            pointer-events: auto;
        }

        .modal-content {
            background: #111827;
            border: 1px solid var(--card-border);
            border-radius: 16px;
            width: 90%;
            max-width: 800px;
            max-height: 90vh;
            display: flex;
            flex-direction: column;
            box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
            overflow: hidden;
        }

        .modal-header {
            padding: 1.25rem 1.5rem;
            border-bottom: 1px solid var(--card-border);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .modal-body {
            padding: 1.5rem;
            overflow-y: auto;
            display: flex;
            flex-direction: column;
            gap: 1rem;
        }

        .form-group {
            display: flex;
            flex-direction: column;
            gap: 0.4rem;
        }

        .form-group label {
            font-size: 0.8rem;
            font-weight: 600;
            color: var(--text-secondary);
        }

        .form-control {
            background: rgba(22, 30, 49, 0.9);
            border: 1px solid var(--card-border);
            border-radius: 8px;
            padding: 0.6rem 0.85rem;
            color: var(--text-primary);
            font-family: inherit;
            font-size: 0.875rem;
            outline: none;
        }

        .form-control:focus {
            border-color: var(--accent-cyan);
        }

        textarea.form-control {
            font-family: 'JetBrains Mono', monospace;
            min-height: 250px;
            resize: vertical;
        }

        .form-row {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 1rem;
        }

        .form-check {
            display: flex;
            align-items: center;
            gap: 0.6rem;
            font-size: 0.85rem;
            cursor: pointer;
        }

        .form-check input[type="checkbox"] {
            width: 16px;
            height: 16px;
            accent-color: var(--accent-cyan);
        }

        .modal-footer {
            padding: 1rem 1.5rem;
            border-top: 1px solid var(--card-border);
            display: flex;
            justify-content: flex-end;
            gap: 0.75rem;
            background: rgba(11, 15, 25, 0.6);
        }

        .modal-tabs {
            display: flex;
            gap: 1rem;
            margin-bottom: 0.5rem;
            border-bottom: 1px solid var(--card-border);
            padding-bottom: 0.5rem;
        }

        /* Toast Notifications */
        .toast-container {
            position: fixed;
            bottom: 1.5rem;
            right: 1.5rem;
            display: flex;
            flex-direction: column;
            gap: 0.75rem;
            z-index: 2000;
        }

        .toast {
            background: #1e293b;
            border: 1px solid var(--card-border);
            color: var(--text-primary);
            padding: 0.85rem 1.25rem;
            border-radius: 10px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.5);
            font-size: 0.85rem;
            display: flex;
            align-items: center;
            gap: 0.75rem;
            animation: slideIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
        }

        @keyframes slideIn {
            from { transform: translateX(100%); opacity: 0; }
            to { transform: translateX(0); opacity: 1; }
        }
    </style>
</head>
<body>
    <nav class="navbar">
        <div class="brand">
            <div class="brand-icon"></div>
            <div class="brand-title">
                <h1>macOS Daemon Manager</h1>
                <p>Launchd & LaunchAgent Controller</p>
            </div>
        </div>
        <div class="sys-status">
            <div id="rootBadge" class="badge badge-user">
                <span class="badge-pulse"></span>
                <span id="rootBadgeText">User Mode</span>
            </div>
            <button class="btn btn-secondary btn-sm" onclick="fetchServices()">⚡ Refresh</button>
            <button class="btn btn-primary" onclick="openCreateModal()">+ Add Service</button>
        </div>
    </nav>

    <div class="container">
        <div class="kpi-grid">
            <div class="kpi-card">
                <div class="kpi-title">Total Services</div>
                <div class="kpi-value" id="kpiTotal">0</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-title">Running Services (PID)</div>
                <div class="kpi-value" style="color: var(--accent-emerald)" id="kpiRunning">0</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-title">User Agents (~/Library)</div>
                <div class="kpi-value" style="color: var(--accent-blue)" id="kpiUser">0</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-title">System Daemons (/Library)</div>
                <div class="kpi-value" style="color: var(--accent-purple)" id="kpiSystem">0</div>
            </div>
        </div>

        <div class="toolbar">
            <div class="tabs">
                <button class="tab active" onclick="setScopeFilter('all', this)">All Services</button>
                <button class="tab" onclick="setScopeFilter('user', this)">User Agents</button>
                <button class="tab" onclick="setScopeFilter('global', this)">Global Agents</button>
                <button class="tab" onclick="setScopeFilter('system', this)">System Daemons</button>
                <button class="tab" onclick="setScopeFilter('running', this)">Active (PID)</button>
            </div>
            <div class="search-box">
                <input type="text" id="searchInput" placeholder="Search label, path or command..." oninput="renderServices()">
            </div>
        </div>

        <div id="serviceGrid" class="service-grid"></div>
    </div>

    <!-- Add / Edit Modal -->
    <div id="serviceModal" class="modal-overlay">
        <div class="modal-content">
            <div class="modal-header">
                <h3 id="modalTitle">Register New Auto-Start Service</h3>
                <button class="btn btn-secondary btn-sm" onclick="closeModal('serviceModal')">✕</button>
            </div>
            <div class="modal-body">
                <div class="modal-tabs">
                    <button class="tab active" id="tabFormBtn" onclick="switchModalTab('form')">Form Editor</button>
                    <button class="tab" id="tabRawBtn" onclick="switchModalTab('raw')">Raw Plist XML</button>
                </div>

                <div id="formTabContent">
                    <div class="form-row" style="margin-bottom: 1rem;">
                        <div class="form-group">
                            <label>Service Label (Identifier)</label>
                            <input type="text" id="formLabel" class="form-control" placeholder="com.user.mydaemon">
                        </div>
                        <div class="form-group">
                            <label>Target Scope</label>
                            <select id="formScope" class="form-control">
                                <option value="user">User Agent (~/Library/LaunchAgents)</option>
                                <option value="global">Global Agent (/Library/LaunchAgents)</option>
                                <option value="system">System Daemon (/Library/LaunchDaemons - Sudo required)</option>
                            </select>
                        </div>
                    </div>

                    <div class="form-group" style="margin-bottom: 1rem;">
                        <label>Executable Command / Program</label>
                        <input type="text" id="formExec" class="form-control" placeholder="/usr/local/bin/node /app/index.js">
                    </div>

                    <div class="form-row" style="margin-bottom: 1rem;">
                        <div class="form-group">
                            <label>Standard Out Log Path</label>
                            <input type="text" id="formStdout" class="form-control" placeholder="/tmp/mydaemon.stdout.log">
                        </div>
                        <div class="form-group">
                            <label>Standard Error Log Path</label>
                            <input type="text" id="formStderr" class="form-control" placeholder="/tmp/mydaemon.stderr.log">
                        </div>
                    </div>

                    <div class="form-row" style="margin-bottom: 1rem;">
                        <div class="form-group">
                            <label>Working Directory</label>
                            <input type="text" id="formWorkdir" class="form-control" placeholder="/Users/dev/project">
                        </div>
                        <div class="form-group">
                            <label>Start Interval (Seconds)</label>
                            <input type="number" id="formInterval" class="form-control" placeholder="60">
                        </div>
                    </div>

                    <div class="form-row">
                        <label class="form-check">
                            <input type="checkbox" id="formRunAtLoad" checked>
                            RunAtLoad (Auto start on system login/boot)
                        </label>
                        <label class="form-check">
                            <input type="checkbox" id="formKeepAlive" checked>
                            KeepAlive (Auto restart if process exits)
                        </label>
                    </div>
                </div>

                <div id="rawTabContent" style="display: none;">
                    <div class="form-group">
                        <label>Plist XML Content (Strict Plist DTD Format)</label>
                        <textarea id="rawXmlArea" class="form-control"></textarea>
                    </div>
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="closeModal('serviceModal')">Cancel</button>
                <button class="btn btn-primary" onclick="saveService()">Save & Register Service</button>
            </div>
        </div>
    </div>

    <!-- Log Viewer Modal -->
    <div id="logModal" class="modal-overlay">
        <div class="modal-content" style="max-width: 900px;">
            <div class="modal-header">
                <h3 id="logModalTitle">Service Log Viewer</h3>
                <button class="btn btn-secondary btn-sm" onclick="closeModal('logModal')">✕</button>
            </div>
            <div class="modal-body">
                <div id="logContent" style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem; background: #090d16; padding: 1rem; border-radius: 8px; border: 1px solid var(--card-border); max-height: 450px; overflow-y: auto; white-space: pre-wrap; color: #a7f3d0;"></div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="closeModal('logModal')">Close</button>
            </div>
        </div>
    </div>

    <div id="toastContainer" class="toast-container"></div>

    <script>
        let allServices = [];
        let currentFilter = 'all';
        let activeModalTab = 'form';

        async function init() {
            await fetchStatus();
            await fetchServices();
        }

        async function fetchStatus() {
            try {
                const res = await fetch('/api/status');
                const data = await res.json();
                const badge = document.getElementById('rootBadge');
                const badgeText = document.getElementById('rootBadgeText');
                if (data.is_root) {
                    badge.className = 'badge badge-root';
                    badgeText.innerText = 'ROOT privileged';
                } else {
                    badge.className = 'badge badge-user';
                    badgeText.innerText = `User Mode (${data.user_name})`;
                }
            } catch (err) {
                console.error('Status fetch error:', err);
            }
        }

        async function fetchServices() {
            try {
                const res = await fetch('/api/services');
                const data = await res.json();
                allServices = data.services || [];
                updateKpis();
                renderServices();
            } catch (err) {
                showToast('Failed to fetch services list', 'error');
            }
        }

        function updateKpis() {
            document.getElementById('kpiTotal').innerText = allServices.length;
            document.getElementById('kpiRunning').innerText = allServices.filter(s => s.pid !== null).length;
            document.getElementById('kpiUser').innerText = allServices.filter(s => s.scope === 'user').length;
            document.getElementById('kpiSystem').innerText = allServices.filter(s => s.scope === 'system_daemon').length;
        }

        function setScopeFilter(filter, el) {
            currentFilter = filter;
            document.querySelectorAll('.toolbar .tab').forEach(t => t.classList.remove('active'));
            el.classList.add('active');
            renderServices();
        }

        function renderServices() {
            const search = document.getElementById('searchInput').value.toLowerCase();
            const grid = document.getElementById('serviceGrid');
            grid.innerHTML = '';

            const filtered = allServices.filter(s => {
                if (currentFilter === 'user' && s.scope !== 'user') return false;
                if (currentFilter === 'global' && s.scope !== 'global_agent') return false;
                if (currentFilter === 'system' && s.scope !== 'system_daemon') return false;
                if (currentFilter === 'running' && s.pid === null) return false;

                if (search) {
                    const matchLabel = s.label.toLowerCase().includes(search);
                    const matchPath = s.plist_path.toLowerCase().includes(search);
                    const matchExec = s.plist_data && s.plist_data.ProgramArguments && s.plist_data.ProgramArguments.join(' ').toLowerCase().includes(search);
                    return matchLabel || matchPath || matchExec;
                }
                return true;
            });

            if (filtered.length === 0) {
                grid.innerHTML = '<div style="grid-column: 1/-1; text-align: center; padding: 3rem; color: var(--text-muted);">No services matched criteria</div>';
                return;
            }

            filtered.forEach(s => {
                const card = document.createElement('div');
                card.className = 'service-card';

                let scopeClass = 'scope-user';
                let scopeLabel = 'User';
                if (s.scope === 'global_agent') { scopeClass = 'scope-global'; scopeLabel = 'Global'; }
                if (s.scope === 'system_daemon') { scopeClass = 'scope-system'; scopeLabel = 'System'; }

                let statusBadge = '<span style="color: var(--text-muted);">● Unloaded</span>';
                if (s.pid !== null) {
                    statusBadge = `<span style="color: var(--accent-emerald); font-weight:700;">● Running (PID: ${s.pid})</span>`;
                } else if (s.is_loaded) {
                    statusBadge = '<span style="color: var(--accent-amber); font-weight:600;">● Loaded (Stopped)</span>';
                }

                const logPath = s.plist_data ? (s.plist_data.StandardOutPath || s.plist_data.StandardErrorPath || '') : '';

                card.innerHTML = `
                    <div>
                        <div class="service-header">
                            <div class="service-title">${escapeHtml(s.label)}</div>
                            <span class="scope-tag ${scopeClass}">${scopeLabel}</span>
                        </div>
                        <div class="service-meta">
                            <div>Status: ${statusBadge}</div>
                            <div class="meta-line" title="${escapeHtml(cmd)}">Exec: ${escapeHtml(cmd)}</div>
                            <div class="meta-line" title="${escapeHtml(s.plist_path)}">Path: ${escapeHtml(s.plist_path)}</div>
                        </div>
                    </div>
                    <div class="service-actions">
                        ${s.pid !== null ? 
                            `<button class="btn btn-secondary btn-sm" onclick="triggerAction('${encodeURIComponent(s.label)}', '${encodeURIComponent(s.scope)}', 'stop')">Stop</button>` :
                            `<button class="btn btn-primary btn-sm" onclick="triggerAction('${encodeURIComponent(s.label)}', '${encodeURIComponent(s.scope)}', 'start')">Start</button>`
                        }
                        ${s.is_loaded ? 
                            `<button class="btn btn-secondary btn-sm" onclick="triggerAction('${encodeURIComponent(s.label)}', '${encodeURIComponent(s.scope)}', 'unload')">Unload</button>` :
                            `<button class="btn btn-secondary btn-sm" onclick="triggerAction('${encodeURIComponent(s.label)}', '${encodeURIComponent(s.scope)}', 'load')">Load</button>`
                        }
                        <button class="btn btn-secondary btn-sm" onclick="openEditModal('${encodeURIComponent(s.label)}', '${encodeURIComponent(s.scope)}')">Edit</button>
                        <button class="btn btn-secondary btn-sm" onclick="viewLog('${encodeURIComponent(logPath)}', '${encodeURIComponent(s.label)}')">Log</button>
                        <button class="btn btn-danger btn-sm" onclick="confirmDelete('${encodeURIComponent(s.label)}', '${encodeURIComponent(s.scope)}')">Delete</button>
                    </div>
                `;
                grid.appendChild(card);
            });
        }

        async function triggerAction(encodedLabel, encodedScope, action) {
            const label = decodeURIComponent(encodedLabel);
            const scope = decodeURIComponent(encodedScope);
            try {
                const res = await fetch(`/api/services/${encodeURIComponent(label)}/action`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ action, scope })
                });
                const data = await res.json();
                if (res.ok) {
                    showToast(data.message, 'success');
                    await fetchServices();
                } else {
                    showToast(data.error || 'Action failed', 'error');
                }
            } catch (err) {
                showToast('Failed to trigger action', 'error');
            }
        }

        async function confirmDelete(encodedLabel, encodedScope) {
            const label = decodeURIComponent(encodedLabel);
            const scope = decodeURIComponent(encodedScope);
            if (confirm(`Are you sure you want to delete service "${label}"? This will unload and remove its plist file.`)) {
                try {
                    const res = await fetch(`/api/services/${encodeURIComponent(label)}?scope=${encodeURIComponent(scope)}`, {
                        method: 'DELETE'
                    });
                    const data = await res.json();
                    if (res.ok) {
                        showToast(data.message, 'success');
                        await fetchServices();
                    } else {
                        showToast(data.error || 'Delete failed', 'error');
                    }
                } catch (err) {
                    showToast('Failed to delete service', 'error');
                }
            }
        }

        function openCreateModal() {
            document.getElementById('modalTitle').innerText = 'Register New Auto-Start Service';
            document.getElementById('formLabel').value = '';
            document.getElementById('formLabel').disabled = false;
            document.getElementById('formExec').value = '';
            document.getElementById('formStdout').value = '';
            document.getElementById('formStderr').value = '';
            document.getElementById('formWorkdir').value = '';
            document.getElementById('formInterval').value = '';
            document.getElementById('rawXmlArea').value = `<?xml version="1.0" encoding="UTF-8"?>\n<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n<plist version="1.0">\n<dict>\n\t<key>Label</key>\n\t<string>com.user.myservice</string>\n\t<key>ProgramArguments</key>\n\t<array>\n\t\t<string>/bin/echo</string>\n\t\t<string>Hello World</string>\n\t</array>\n\t<key>RunAtLoad</key>\n\t<true/>\n</dict>\n</plist>`;
            switchModalTab('form');
            document.getElementById('serviceModal').classList.add('active');
        }

        async function openEditModal(encodedLabel, encodedScope) {
            const label = decodeURIComponent(encodedLabel);
            const scope = decodeURIComponent(encodedScope);
            try {
                const res = await fetch(`/api/services/${encodeURIComponent(label)}?scope=${encodeURIComponent(scope)}`);
                const data = await res.json();
                if (!res.ok) throw new Error(data.error);

                const item = data.item;
                document.getElementById('modalTitle').innerText = `Edit Service: ${item.label}`;
                document.getElementById('formLabel').value = item.label;
                document.getElementById('formLabel').disabled = true;
                document.getElementById('formScope').value = item.scope === 'system_daemon' ? 'system' : (item.scope === 'global_agent' ? 'global' : 'user');

                if (item.plist_data) {
                    const p = item.plist_data;
                    document.getElementById('formExec').value = p.ProgramArguments ? p.ProgramArguments.join(' ') : (p.Program || '');
                    document.getElementById('formStdout').value = p.StandardOutPath || '';
                    document.getElementById('formStderr').value = p.StandardErrorPath || '';
                    document.getElementById('formWorkdir').value = p.WorkingDirectory || '';
                    document.getElementById('formInterval').value = p.StartInterval || '';
                    document.getElementById('formRunAtLoad').checked = p.RunAtLoad !== false;
                    document.getElementById('formKeepAlive').checked = !!p.KeepAlive;
                }

                document.getElementById('rawXmlArea').value = data.raw_xml;
                switchModalTab('form');
                document.getElementById('serviceModal').classList.add('active');
            } catch (err) {
                showToast(err.message || 'Failed to fetch service detail', 'error');
            }
        }

        function switchModalTab(tab) {
            activeModalTab = tab;
            if (tab === 'form') {
                document.getElementById('tabFormBtn').classList.add('active');
                document.getElementById('tabRawBtn').classList.remove('active');
                document.getElementById('formTabContent').style.display = 'block';
                document.getElementById('rawTabContent').style.display = 'none';
            } else {
                document.getElementById('tabRawBtn').classList.add('active');
                document.getElementById('tabFormBtn').classList.remove('active');
                document.getElementById('formTabContent').style.display = 'none';
                document.getElementById('rawTabContent').style.display = 'block';
            }
        }

        async function saveService() {
            const label = document.getElementById('formLabel').value.trim();
            const scope = document.getElementById('formScope').value;

            if (!label) {
                showToast('Service Label is required', 'error');
                return;
            }

            if (activeModalTab === 'raw') {
                const xml_content = document.getElementById('rawXmlArea').value;
                try {
                    const res = await fetch('/api/services/raw', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ label, scope, xml_content })
                    });
                    const data = await res.json();
                    if (res.ok) {
                        showToast(data.message, 'success');
                        closeModal('serviceModal');
                        await fetchServices();
                    } else {
                        showToast(data.error || 'Failed to save raw XML', 'error');
                    }
                } catch (err) {
                    showToast('Failed to connect to server', 'error');
                }
            } else {
                const execRaw = document.getElementById('formExec').value.trim();
                if (!execRaw) {
                    showToast('Executable command is required', 'error');
                    return;
                }

                const execParts = execRaw.split(' ').filter(p => p.length > 0);
                const exec = execParts[0];
                const args = execParts.slice(1);

                const payload = {
                    label,
                    scope,
                    exec,
                    args: args.length > 0 ? args : null,
                    run_at_load: document.getElementById('formRunAtLoad').checked,
                    keep_alive: document.getElementById('formKeepAlive').checked,
                    stdout_path: document.getElementById('formStdout').value.trim() || null,
                    stderr_path: document.getElementById('formStderr').value.trim() || null,
                    workdir: document.getElementById('formWorkdir').value.trim() || null,
                    interval: parseInt(document.getElementById('formInterval').value) || null,
                };

                try {
                    const res = await fetch('/api/services', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify(payload)
                    });
                    const data = await res.json();
                    if (res.ok) {
                        showToast(data.message, 'success');
                        closeModal('serviceModal');
                        await fetchServices();
                    } else {
                        showToast(data.error || 'Failed to save service', 'error');
                    }
                } catch (err) {
                    showToast('Failed to save service', 'error');
                }
            }
        }

        async function viewLog(encodedPath, encodedLabel) {
            const path = decodeURIComponent(encodedPath || '');
            const label = decodeURIComponent(encodedLabel || '');

            if (!path) {
                document.getElementById('logModalTitle').innerText = `Service Log Viewer: ${label}`;
                document.getElementById('logContent').innerText = `(No log file path [StandardOutPath / StandardErrorPath] configured for ${label})`;
                document.getElementById('logModal').classList.add('active');
                return;
            }

            try {
                const res = await fetch(`/api/logs?path=${encodeURIComponent(path)}&lines=200`);
                const data = await res.json();
                document.getElementById('logModalTitle').innerText = `Log Viewer: ${path}`;
                document.getElementById('logContent').innerText = data.content || '(Log file empty)';
                document.getElementById('logModal').classList.add('active');
            } catch (err) {
                showToast('Failed to fetch log file', 'error');
            }
        }

        function closeModal(modalId) {
            document.getElementById(modalId).classList.remove('active');
        }

        function showToast(msg, type = 'info') {
            const container = document.getElementById('toastContainer');
            const toast = document.createElement('div');
            toast.className = 'toast';
            if (type === 'error') toast.style.borderColor = 'rgba(244, 63, 94, 0.5)';
            if (type === 'success') toast.style.borderColor = 'rgba(16, 185, 129, 0.5)';
            toast.innerText = msg;
            container.appendChild(toast);
            setTimeout(() => toast.remove(), 4000);
        }

        function escapeHtml(str) {
            if (!str) return '';
            return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#039;");
        }

        window.onload = init;
    </script>
</body>
</html>
"#;
