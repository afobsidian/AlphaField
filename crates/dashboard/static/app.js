const API_BASE = '/api';

// ============================================================================
// STATE
// ============================================================================

let allSymbols = [];
const POPULAR_SYMBOLS = ['BTC', 'ETH', 'SOL', 'XRP', 'ADA', 'DOGE', 'AVAX', 'DOT', 'LINK', 'MATIC'];

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

const formatMoney = (value) => new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD'
}).format(value);

const formatPercent = (value) => `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;

const colorClass = (value) => value >= 0 ? 'positive' : 'negative';

// ============================================================================
// TAB NAVIGATION
// ============================================================================

function switchTab(tabName) {
    // Hide all views
    document.querySelectorAll('.view-section').forEach(v => v.style.display = 'none');
    document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));

    // Show selected
    document.getElementById(`${tabName}-view`).style.display = 'block';
    event.currentTarget.classList.add('active');

    // Load data for tab
    if (tabName === 'data') {
        loadSymbols();
        loadTradingPairs();
    }
    if (tabName === 'backtest') populateSymbolDropdown();
    if (tabName === 'sentiment') loadSentimentData();
}

// ============================================================================
// SYMBOL SEARCH & AUTOCOMPLETE
// ============================================================================

async function loadTradingPairs() {
    try {
        const res = await fetch(`${API_BASE}/data/pairs`);
        allSymbols = await res.json();
        showSymbolDropdown('');  // Show popular symbols immediately
    } catch (e) {
        allSymbols = POPULAR_SYMBOLS;
    }
}

function showSymbolDropdown(query) {
    const dropdown = document.getElementById('symbol-dropdown');
    const input = document.getElementById('fetch-symbol');

    if (!dropdown) return;

    // Filter and limit results
    let filtered = allSymbols.filter(s =>
        s.toLowerCase().includes(query.toLowerCase())
    ).slice(0, 20);

    // If no query, show popular first
    if (!query) {
        const popular = allSymbols.filter(s => POPULAR_SYMBOLS.includes(s));
        const others = allSymbols.filter(s => !POPULAR_SYMBOLS.includes(s)).slice(0, 10);
        filtered = [...popular, ...others];
    }

    if (filtered.length === 0) {
        dropdown.classList.remove('active');
        return;
    }

    dropdown.innerHTML = filtered.map(s => {
        const isPopular = POPULAR_SYMBOLS.includes(s);
        return `
            <div class="symbol-option ${isPopular ? 'popular' : ''}" onclick="selectSymbol('${s}')">
                <span class="symbol-name">${s}</span>
                ${isPopular ? '<span class="symbol-badge">Popular</span>' : ''}
            </div>
        `;
    }).join('');

    dropdown.classList.add('active');
}

function selectSymbol(symbol) {
    document.getElementById('fetch-symbol').value = symbol;
    document.getElementById('symbol-dropdown').classList.remove('active');
}

function initSymbolSearch() {
    const input = document.getElementById('fetch-symbol');
    const dropdown = document.getElementById('symbol-dropdown');

    if (!input || !dropdown) return;

    // Show dropdown on focus
    input.addEventListener('focus', () => {
        showSymbolDropdown(input.value);
    });

    // Filter on input
    input.addEventListener('input', (e) => {
        showSymbolDropdown(e.target.value);
    });

    // Hide on click outside
    document.addEventListener('click', (e) => {
        if (!e.target.closest('.autocomplete-wrapper')) {
            dropdown.classList.remove('active');
        }
    });
}

// ============================================================================
// DATA MANAGEMENT
// ============================================================================

async function loadSymbols() {
    const container = document.getElementById('symbols-list');
    container.innerHTML = '<div class="loading">Loading...</div>';

    try {
        const res = await fetch(`${API_BASE}/data/symbols`);
        const symbols = await res.json();

        if (symbols.length === 0) {
            container.innerHTML = '<div class="loading">No data cached. Fetch some data to get started.</div>';
            return;
        }

        container.innerHTML = symbols.map(s => `
            <div class="symbol-item">
                <div class="symbol-info">
                    <h4>${s.symbol} / ${s.timeframe}</h4>
                    <p>${s.first_bar || 'Unknown'} → ${s.last_bar || 'Unknown'}</p>
                </div>
                <div class="symbol-meta">
                    <div class="bars">${s.bar_count.toLocaleString()} bars</div>
                    <button class="btn btn-danger" onclick="deleteSymbol('${s.symbol}', '${s.timeframe}')">Delete</button>
                </div>
            </div>
        `).join('');
    } catch (e) {
        container.innerHTML = '<div class="loading">Failed to load symbols</div>';
    }
}

async function fetchData(e) {
    e.preventDefault();
    const btn = e.target.querySelector('button');
    const statusEl = document.getElementById('fetch-status');

    btn.disabled = true;
    btn.innerHTML = '<span>⏳</span> Fetching...';
    statusEl.className = 'status-message';
    statusEl.textContent = '';

    try {
        const interval = document.getElementById('fetch-interval').value;
        const days = parseInt(document.getElementById('fetch-days').value);

        // Calculate number of bars based on interval and days
        let barsPerDay = 24; // Default for 1h
        if (interval === '4h') barsPerDay = 6;
        if (interval === '1d') barsPerDay = 1;
        const limit = days * barsPerDay;

        const res = await fetch(`${API_BASE}/data/fetch`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                symbol: document.getElementById('fetch-symbol').value,
                interval: interval,
                limit: limit
            })
        });

        const data = await res.json();

        if (data.success) {
            statusEl.className = 'status-message success';
            statusEl.textContent = `✓ ${data.message}`;
            loadSymbols();
        } else {
            statusEl.className = 'status-message error';
            statusEl.textContent = `✗ ${data.message}`;
        }
    } catch (e) {
        statusEl.className = 'status-message error';
        statusEl.textContent = `✗ Network error`;
    }

    btn.disabled = false;
    btn.innerHTML = '<span>⬇️</span> Fetch Data';
}

async function deleteSymbol(symbol, interval) {
    if (!confirm(`Delete all ${symbol} ${interval} data?`)) return;

    try {
        await fetch(`${API_BASE}/data/${symbol}/${interval}`, { method: 'DELETE' });
        loadSymbols();
    } catch (e) {
        alert('Failed to delete');
    }
}

// ============================================================================
// BACKTEST
// ============================================================================

async function populateSymbolDropdown() {
    const select = document.getElementById('bt-symbol');

    try {
        const res = await fetch(`${API_BASE}/data/symbols`);
        const symbols = await res.json();

        if (symbols.length === 0) {
            select.innerHTML = '<option value="">No data - fetch first</option>';
            return;
        }

        // Deduplicate symbols
        const uniqueSymbols = [...new Set(symbols.map(s => s.symbol))];
        select.innerHTML = uniqueSymbols.map(s => `<option value="${s}">${s}</option>`).join('');
    } catch (e) {
        select.innerHTML = '<option value="BTC">BTC</option>';
    }
}

function updateParams() {
    const strategy = document.getElementById('bt-strategy').value;
    const container = document.getElementById('strategy-params');

    const params = {
        GoldenCross: `
            <div class="form-group">
                <label>Fast Period</label>
                <input type="number" name="fast_period" value="10">
            </div>
            <div class="form-group">
                <label>Slow Period</label>
                <input type="number" name="slow_period" value="30">
            </div>
        `,
        Rsi: `
            <div class="form-group">
                <label>Period</label>
                <input type="number" name="period" value="14">
            </div>
            <div class="form-group">
                <label>Lower Bound</label>
                <input type="number" name="lower_bound" value="30">
            </div>
            <div class="form-group">
                <label>Upper Bound</label>
                <input type="number" name="upper_bound" value="70">
            </div>
        `,
        MeanReversion: `
            <div class="form-group">
                <label>Period</label>
                <input type="number" name="period" value="20">
            </div>
            <div class="form-group">
                <label>Std Dev</label>
                <input type="number" step="0.1" name="std_dev" value="2.0">
            </div>
        `,
        Momentum: `
            <div class="form-group">
                <label>EMA Period</label>
                <input type="number" name="ema_period" value="50">
            </div>
            <div class="form-group">
                <label>MACD Fast</label>
                <input type="number" name="macd_fast" value="12">
            </div>
            <div class="form-group">
                <label>MACD Slow</label>
                <input type="number" name="macd_slow" value="26">
            </div>
            <div class="form-group">
                <label>MACD Signal</label>
                <input type="number" name="macd_signal" value="9">
            </div>
        `
    };

    container.innerHTML = params[strategy] || '';
}

async function runBacktest(e) {
    e.preventDefault();
    const btn = e.target.querySelector('button');
    btn.disabled = true;
    btn.innerHTML = '<span>⏳</span> Running...';

    try {
        const strategy = document.getElementById('bt-strategy').value;
        const symbol = document.getElementById('bt-symbol').value;
        const days = parseInt(document.getElementById('bt-days').value);
        const startDate = document.getElementById('bt-start-date').value;
        const endDate = document.getElementById('bt-end-date').value;

        // Collect strategy params
        const params = {};
        document.querySelectorAll('.strategy-param').forEach(input => {
            params[input.dataset.param] = parseFloat(input.value);
        });

        const requestBody = {
            strategy,
            symbol,
            interval: '1h',
            days,
            params,
            include_benchmark: true
        };

        if (startDate) requestBody.start_date = new Date(startDate).toISOString();
        if (endDate) requestBody.end_date = new Date(endDate).toISOString();

        const res = await fetch(`${API_BASE}/backtest/run`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(requestBody)
        });

        const data = await res.json();

        // Update core metrics
        updateMetric('bt-return', data.metrics.total_return * 100, '%');
        updateMetric('bt-cagr', data.metrics.cagr * 100, '%');
        document.getElementById('bt-sharpe').textContent = data.metrics.sharpe_ratio.toFixed(2);
        document.getElementById('bt-sortino').textContent = data.metrics.sortino_ratio.toFixed(2);
        updateMetric('bt-drawdown', -data.metrics.max_drawdown * 100, '%', true);
        document.getElementById('bt-calmar').textContent = data.metrics.calmar_ratio.toFixed(2);

        // Update benchmark comparison
        if (data.benchmark) {
            updateMetric('bt-alpha', data.benchmark.comparison.alpha * 100, '%');
            document.getElementById('bt-beta').textContent = data.benchmark.comparison.beta.toFixed(2);
            updateMetric('bt-excess', data.benchmark.comparison.excess_return * 100, '%');
            document.getElementById('bt-correlation').textContent = data.benchmark.comparison.correlation.toFixed(2);
        }

        // Update trade stats
        document.getElementById('bt-trades').textContent = data.trade_summary.total_trades;
        updateMetric('bt-winrate', data.trade_summary.winning_trades / Math.max(1, data.trade_summary.total_trades) * 100, '%');
        document.getElementById('bt-pf').textContent = data.metrics.profit_factor.toFixed(2);
        document.getElementById('bt-duration').textContent = `${data.trade_summary.avg_trade_duration_hours.toFixed(1)}h`;
        document.getElementById('bt-winstreak').textContent = data.trade_summary.longest_winning_streak;
        document.getElementById('bt-lossstreak').textContent = data.trade_summary.longest_losing_streak;

        // Update data status
        if (data.data_status) {
            const ds = data.data_status;
            const sourceIcon = ds.source === 'cache' ? '💾' : '🌐';
            const cacheNote = ds.cached_after ? ' (cached)' : '';
            document.getElementById('bt-data-source').innerHTML =
                `${sourceIcon} ${ds.bars_loaded} bars from ${ds.source}${cacheNote}`;

            if (ds.date_range_start && ds.date_range_end) {
                const start = new Date(ds.date_range_start).toLocaleDateString();
                const end = new Date(ds.date_range_end).toLocaleDateString();
                document.getElementById('bt-date-range').textContent = `${start} — ${end}`;
            }
        }

        // Update execution time
        document.getElementById('bt-exec-time').textContent = data.execution_time_ms;

        // Render Plotly Charts
        renderEquityChart(data.equity_curve, data.benchmark?.curve);
        renderDrawdownChart(data.drawdown_curve);
        renderRollingChart(data.rolling_stats);
        renderMonthlyReturnsChart(data.monthly_returns);
        renderTradesChart(data.trades);

        // Fetch and display market sentiment (Fear & Greed Index) for the backtest period
        await fetchBacktestSentiment(days);

        // Display asset sentiment from backtest response
        if (data.asset_sentiment) {
            displayAssetSentiment(data.asset_sentiment);
        }

    } catch (e) {
        alert('Backtest failed: ' + e.message);
    }

    btn.disabled = false;
    btn.innerHTML = '<span>▶️</span> Run Backtest';
}

function updateMetric(id, value, suffix = '', invert = false) {
    const el = document.getElementById(id);
    el.textContent = `${value >= 0 ? '+' : ''}${value.toFixed(2)}${suffix}`;
    el.className = `metric-value ${invert ? (value <= 0 ? 'positive' : 'negative') : (value >= 0 ? 'positive' : 'negative')}`;
}

function renderEquityChart(equityCurve, benchmarkCurve) {
    const timestamps = equityCurve.map(p => new Date(p.timestamp));
    const equity = equityCurve.map(p => p.equity);

    const traces = [{
        x: timestamps,
        y: equity,
        type: 'scatter',
        mode: 'lines',
        name: 'Strategy',
        line: { color: '#3b82f6', width: 2 }
    }];

    if (benchmarkCurve && benchmarkCurve.length > 0) {
        traces.push({
            x: benchmarkCurve.map(p => new Date(p.timestamp)),
            y: benchmarkCurve.map(p => p.equity),
            type: 'scatter',
            mode: 'lines',
            name: 'BTC Buy & Hold',
            line: { color: '#6b7280', width: 1, dash: 'dot' }
        });
    }

    Plotly.newPlot('equity-chart', traces, {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        margin: { t: 20, r: 20, b: 40, l: 60 },
        xaxis: { gridcolor: '#334155' },
        yaxis: { gridcolor: '#334155', title: 'Equity ($)' },
        legend: { x: 0, y: 1.1, orientation: 'h' },
        hovermode: 'x unified'
    }, { responsive: true });
}

function renderDrawdownChart(drawdownCurve) {
    if (!drawdownCurve || drawdownCurve.length === 0) return;

    Plotly.newPlot('drawdown-chart', [{
        x: drawdownCurve.map(p => new Date(p.timestamp)),
        y: drawdownCurve.map(p => -p.drawdown * 100),
        type: 'scatter',
        mode: 'lines',
        fill: 'tozeroy',
        name: 'Drawdown',
        line: { color: '#ef4444', width: 1 },
        fillcolor: 'rgba(239, 68, 68, 0.3)'
    }], {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        margin: { t: 20, r: 20, b: 40, l: 60 },
        xaxis: { gridcolor: '#334155' },
        yaxis: { gridcolor: '#334155', title: 'Drawdown (%)', range: [null, 0] },
        hovermode: 'x'
    }, { responsive: true });
}

function renderRollingChart(rollingStats) {
    if (!rollingStats || !rollingStats.rolling_sharpe || rollingStats.rolling_sharpe.length === 0) return;

    Plotly.newPlot('rolling-chart', [{
        x: rollingStats.rolling_sharpe.map(p => new Date(p[0])),
        y: rollingStats.rolling_sharpe.map(p => p[1]),
        type: 'scatter',
        mode: 'lines',
        name: 'Rolling Sharpe',
        line: { color: '#10b981', width: 2 }
    }], {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        margin: { t: 20, r: 20, b: 40, l: 60 },
        xaxis: { gridcolor: '#334155' },
        yaxis: { gridcolor: '#334155', title: 'Sharpe Ratio' },
        shapes: [{
            type: 'line',
            x0: 0, x1: 1, xref: 'paper',
            y0: 0, y1: 0,
            line: { color: '#6b7280', dash: 'dash' }
        }],
        hovermode: 'x'
    }, { responsive: true });
}

function renderMonthlyReturnsChart(monthlyReturns) {
    if (!monthlyReturns || monthlyReturns.length === 0) {
        document.getElementById('monthly-chart').innerHTML = '<p style="color:#94a3b8;text-align:center;padding:20px;">No monthly data</p>';
        return;
    }

    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
    const years = [...new Set(monthlyReturns.map(m => m.year))].sort();

    const z = years.map(year => {
        return months.map((_, monthIdx) => {
            const found = monthlyReturns.find(m => m.year === year && m.month === monthIdx + 1);
            return found ? found.return_pct * 100 : null;
        });
    });

    Plotly.newPlot('monthly-chart', [{
        z: z,
        x: months,
        y: years.map(String),
        type: 'heatmap',
        colorscale: [
            [0, '#ef4444'],
            [0.5, '#1e293b'],
            [1, '#10b981']
        ],
        zmid: 0,
        text: z.map(row => row.map(v => v !== null ? `${v.toFixed(1)}%` : '')),
        texttemplate: '%{text}',
        textfont: { size: 10, color: '#fff' },
        hovertemplate: '%{y} %{x}: %{z:.1f}%<extra></extra>'
    }], {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        margin: { t: 20, r: 20, b: 40, l: 60 },
        xaxis: { side: 'top' },
        yaxis: { autorange: 'reversed' }
    }, { responsive: true });
}

function renderTradesChart(trades) {
    if (!trades || trades.length === 0) {
        document.getElementById('trades-chart').innerHTML = '<p style="color:#94a3b8;text-align:center;padding:20px;">No trades</p>';
        return;
    }

    const pnls = trades.map(t => t.pnl);
    const colors = trades.map(t => t.pnl >= 0 ? '#10b981' : '#ef4444');

    Plotly.newPlot('trades-chart', [{
        x: pnls,
        type: 'histogram',
        marker: { color: '#3b82f6' },
        nbinsx: 20
    }], {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        margin: { t: 20, r: 20, b: 40, l: 60 },
        xaxis: { gridcolor: '#334155', title: 'P&L ($)' },
        yaxis: { gridcolor: '#334155', title: 'Count' },
        bargap: 0.1
    }, { responsive: true });
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

async function checkHealth() {
    try {
        const res = await fetch(`${API_BASE}/health`);
        const data = await res.json();

        const badge = document.getElementById('db-status');
        if (data.database === 'connected') {
            badge.className = 'status-badge connected';
            badge.innerHTML = '<span class="status-dot"></span>DB Connected';
        } else {
            badge.className = 'status-badge disconnected';
            badge.innerHTML = '<span class="status-dot"></span>DB Offline';
        }
    } catch (e) {
        document.getElementById('db-status').innerHTML = '<span class="status-dot"></span>Offline';
    }
}

// ============================================================================
// LIVE VIEW
// ============================================================================

let equityChart;

function initEquityChart() {
    const ctx = document.getElementById('equityChart');
    if (!ctx) return;

    equityChart = new Chart(ctx.getContext('2d'), {
        type: 'line',
        data: { labels: ['Day 1', 'Day 2', 'Day 3'], datasets: [{ label: 'Equity', data: [100000, 101200, 102500], borderColor: '#3b82f6', backgroundColor: 'rgba(59, 130, 246, 0.1)', tension: 0.4, fill: true }] },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: { legend: { display: false } },
            scales: {
                y: { grid: { color: '#334155' }, ticks: { color: '#94a3b8' } },
                x: { grid: { display: false }, ticks: { color: '#94a3b8' } }
            }
        }
    });
}

// ============================================================================
// WEBSOCKET CLIENT
// ============================================================================

let ws = null;
let wsReconnectAttempts = 0;
const WS_MAX_RECONNECT_ATTEMPTS = 10;
const WS_RECONNECT_DELAY_MS = 2000;

function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/api/ws`;

    console.log('Connecting to WebSocket:', wsUrl);
    updateWsStatus('connecting');

    try {
        ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            console.log('WebSocket connected');
            wsReconnectAttempts = 0;
            updateWsStatus('connected');
            addLog('info', 'Connected to real-time updates');

            // Request initial state
            ws.send(JSON.stringify({ command: 'Snapshot' }));
        };

        ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                handleWsMessage(msg);
            } catch (e) {
                console.error('Failed to parse WebSocket message:', e);
            }
        };

        ws.onclose = (event) => {
            console.log('WebSocket closed:', event.code, event.reason);
            updateWsStatus('disconnected');

            // Auto-reconnect
            if (wsReconnectAttempts < WS_MAX_RECONNECT_ATTEMPTS) {
                wsReconnectAttempts++;
                const delay = WS_RECONNECT_DELAY_MS * wsReconnectAttempts;
                console.log(`Reconnecting in ${delay}ms (attempt ${wsReconnectAttempts})`);
                setTimeout(connectWebSocket, delay);
            } else {
                addLog('error', 'Connection lost. Please refresh the page.');
            }
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            updateWsStatus('error');
        };
    } catch (e) {
        console.error('Failed to create WebSocket:', e);
        updateWsStatus('error');
    }
}

function handleWsMessage(msg) {
    switch (msg.type) {
        case 'Portfolio':
            updatePortfolioUI(msg.data);
            break;
        case 'Position':
            updatePositionUI(msg.data);
            break;
        case 'Positions':
            updateAllPositionsUI(msg.data);
            break;
        case 'Trade':
            addTradeToHistory(msg.data);
            break;
        case 'Log':
            addLog(msg.data.level, msg.data.message);
            break;
        case 'EngineStatus':
            updateEngineStatusUI(msg.data);
            break;
        case 'Heartbeat':
            // Connection is alive
            break;
        default:
            console.log('Unknown message type:', msg.type);
    }
}

function updateWsStatus(status) {
    const badge = document.getElementById('ws-status');
    if (!badge) return;

    const statusMap = {
        connecting: { class: 'status-badge connecting', text: '⟳ Connecting...' },
        connected: { class: 'status-badge connected', text: '● Live' },
        disconnected: { class: 'status-badge disconnected', text: '○ Disconnected' },
        error: { class: 'status-badge error', text: '✕ Error' }
    };

    const s = statusMap[status] || statusMap.disconnected;
    badge.className = s.class;
    badge.textContent = s.text;
}

function updatePortfolioUI(data) {
    const el = (id) => document.getElementById(id);

    if (el('live-total-value')) el('live-total-value').textContent = formatMoney(data.total_value);
    if (el('live-cash')) el('live-cash').textContent = formatMoney(data.cash);
    if (el('live-positions-value')) el('live-positions-value').textContent = formatMoney(data.positions_value);
    if (el('live-pnl')) {
        el('live-pnl').textContent = formatMoney(data.pnl);
        el('live-pnl').className = `metric-value ${colorClass(data.pnl)}`;
    }
}

function updatePositionUI(position) {
    const row = document.getElementById(`position-${position.symbol}`);
    if (row) {
        row.querySelector('.pos-price').textContent = formatMoney(position.current_price);
        row.querySelector('.pos-pnl').textContent = formatMoney(position.pnl);
        row.querySelector('.pos-pnl').className = `pos-pnl ${colorClass(position.pnl)}`;
    }
}

function updateAllPositionsUI(positions) {
    const container = document.getElementById('live-positions-table');
    if (!container) return;

    if (positions.length === 0) {
        container.innerHTML = '<tr><td colspan="6" class="empty">No open positions</td></tr>';
        return;
    }

    container.innerHTML = positions.map(p => `
        <tr id="position-${p.symbol}">
            <td>${p.symbol}</td>
            <td>${p.quantity.toFixed(4)}</td>
            <td>${formatMoney(p.entry_price)}</td>
            <td class="pos-price">${formatMoney(p.current_price)}</td>
            <td class="pos-pnl ${colorClass(p.pnl)}">${formatMoney(p.pnl)}</td>
            <td class="${colorClass(p.pnl_percent)}">${formatPercent(p.pnl_percent)}</td>
        </tr>
    `).join('');
}

function addTradeToHistory(trade) {
    const container = document.getElementById('trade-history');
    if (!container) return;

    const tradeTime = new Date(trade.timestamp).toLocaleTimeString();
    const sideClass = trade.side === 'Buy' ? 'positive' : 'negative';

    const row = document.createElement('tr');
    row.innerHTML = `
        <td>${tradeTime}</td>
        <td>${trade.symbol}</td>
        <td class="${sideClass}">${trade.side}</td>
        <td>${trade.quantity.toFixed(4)}</td>
        <td>${formatMoney(trade.price)}</td>
        <td class="${colorClass(trade.pnl || 0)}">${trade.pnl ? formatMoney(trade.pnl) : '-'}</td>
    `;

    // Add to top of list
    container.insertBefore(row, container.firstChild);

    // Limit to 50 trades
    while (container.children.length > 50) {
        container.removeChild(container.lastChild);
    }
}

function updateEngineStatusUI(status) {
    const startBtn = document.getElementById('btn-start');
    const stopBtn = document.getElementById('btn-stop');
    const statusEl = document.getElementById('engine-status');

    if (startBtn) startBtn.disabled = status.running;
    if (stopBtn) stopBtn.disabled = !status.running;

    if (statusEl) {
        statusEl.className = status.running ? 'engine-running' : 'engine-stopped';
        statusEl.textContent = status.running
            ? `Running: ${status.strategy || 'Unknown'} (${status.mode})`
            : 'Stopped';
    }
}

function addLog(level, message) {
    const console = document.getElementById('log-console');
    if (!console) return;

    const time = new Date().toLocaleTimeString();
    const entry = document.createElement('div');
    entry.className = `log-entry log-${level}`;
    entry.innerHTML = `<span class="log-time">${time}</span> <span class="log-msg">${message}</span>`;

    console.appendChild(entry);
    console.scrollTop = console.scrollHeight;

    // Limit to 100 entries
    while (console.children.length > 100) {
        console.removeChild(console.firstChild);
    }
}

// Control panel actions
function startTrading() {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
        addLog('error', 'Not connected');
        return;
    }

    const strategy = document.getElementById('live-strategy')?.value || 'GoldenCross';
    const mode = document.getElementById('live-mode')?.value || 'paper';

    ws.send(JSON.stringify({ command: 'Start', strategy, mode }));
}

function stopTrading() {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
        addLog('error', 'Not connected');
        return;
    }

    ws.send(JSON.stringify({ command: 'Stop' }));
}

function panicClose() {
    if (!confirm('⚠️ PANIC CLOSE: This will close ALL positions immediately. Continue?')) {
        return;
    }

    if (!ws || ws.readyState !== WebSocket.OPEN) {
        addLog('error', 'Not connected');
        return;
    }

    ws.send(JSON.stringify({ command: 'PanicClose' }));
    addLog('warn', 'PANIC CLOSE initiated');
}

// ============================================================================
// SENTIMENT ANALYSIS
// ============================================================================

async function loadSentimentData() {
    // Load current sentiment
    loadCurrentSentiment();
    // Load history
    loadSentimentHistory();
}

async function loadCurrentSentiment() {
    try {
        const res = await fetch(`${API_BASE}/sentiment/current`);
        const data = await res.json();

        if (data.success && data.data) {
            const sent = data.data;

            // Update gauge value
            const gaugeValue = document.getElementById('gauge-value');
            const gaugeLabel = document.getElementById('gauge-label');
            const gaugeMarker = document.getElementById('gauge-marker');

            gaugeValue.textContent = sent.value;
            gaugeLabel.textContent = sent.classification;
            gaugeMarker.style.left = `${sent.value}%`;

            // Update gauge color class based on sentiment
            gaugeValue.className = 'gauge-value';
            if (sent.is_fear) {
                gaugeValue.classList.add('fear');
            } else if (sent.is_greed) {
                gaugeValue.classList.add('greed');
            } else {
                gaugeValue.classList.add('neutral');
            }

            // Update timestamp
            const updated = new Date(sent.timestamp * 1000).toLocaleString();
            document.getElementById('sentiment-updated').textContent = `Last updated: ${updated}`;
        }
    } catch (e) {
        console.error('Failed to load current sentiment:', e);
    }
}

async function loadSentimentHistory() {
    const days = document.getElementById('sentiment-days')?.value || 30;

    try {
        const res = await fetch(`${API_BASE}/sentiment/history?days=${days}`);
        const data = await res.json();

        if (data.success) {
            // Update stats
            if (data.stats) {
                document.getElementById('sent-avg').textContent = data.stats.average.toFixed(1);
                document.getElementById('sent-min').textContent = data.stats.min;
                document.getElementById('sent-max').textContent = data.stats.max;
                document.getElementById('sent-sma7').textContent = data.stats.sma_7?.toFixed(1) || '--';
                document.getElementById('sent-momentum').textContent = data.stats.momentum_7 !== null
                    ? `${data.stats.momentum_7 >= 0 ? '+' : ''}${data.stats.momentum_7.toFixed(1)}`
                    : '--';
                document.getElementById('sent-fear-days').textContent = data.stats.days_in_fear;

                // Update zone bars
                const total = data.stats.days_in_fear + data.stats.days_in_greed + data.stats.days_neutral;
                if (total > 0) {
                    const fearPct = (data.stats.days_in_fear / total * 100).toFixed(0);
                    const neutralPct = (data.stats.days_neutral / total * 100).toFixed(0);
                    const greedPct = (data.stats.days_in_greed / total * 100).toFixed(0);

                    document.getElementById('zone-fear').style.width = `${fearPct}%`;
                    document.getElementById('zone-neutral').style.width = `${neutralPct}%`;
                    document.getElementById('zone-greed').style.width = `${greedPct}%`;

                    document.getElementById('zone-fear-pct').textContent = `${fearPct}%`;
                    document.getElementById('zone-neutral-pct').textContent = `${neutralPct}%`;
                    document.getElementById('zone-greed-pct').textContent = `${greedPct}%`;
                }
            }

            // Render chart
            renderSentimentChart(data.data);
        }
    } catch (e) {
        console.error('Failed to load sentiment history:', e);
    }
}

function renderSentimentChart(sentimentData) {
    if (!sentimentData || sentimentData.length === 0) {
        document.getElementById('sentiment-chart').innerHTML = '<p style="color:#94a3b8;text-align:center;padding:60px;">No sentiment data available</p>';
        return;
    }

    // Sort by timestamp ascending for chart
    const sorted = [...sentimentData].sort((a, b) => a.timestamp - b.timestamp);

    const timestamps = sorted.map(d => new Date(d.timestamp * 1000));
    const values = sorted.map(d => d.value);

    // Color based on value
    const colors = sorted.map(d => {
        if (d.value <= 25) return '#ef4444';      // Extreme Fear - red
        if (d.value <= 45) return '#f97316';      // Fear - orange
        if (d.value <= 55) return '#94a3b8';      // Neutral - gray
        if (d.value <= 75) return '#84cc16';      // Greed - light green
        return '#22c55e';                          // Extreme Greed - green
    });

    Plotly.newPlot('sentiment-chart', [
        {
            x: timestamps,
            y: values,
            type: 'scatter',
            mode: 'lines+markers',
            name: 'Fear & Greed',
            line: { color: '#3b82f6', width: 2 },
            marker: { color: colors, size: 6 }
        }
    ], {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        margin: { t: 20, r: 20, b: 40, l: 60 },
        xaxis: { gridcolor: '#334155' },
        yaxis: {
            gridcolor: '#334155',
            title: 'Fear & Greed Index',
            range: [0, 100]
        },
        shapes: [
            // Extreme Fear zone (0-25)
            { type: 'rect', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: 0, y1: 25, fillcolor: 'rgba(239, 68, 68, 0.1)', line: { width: 0 } },
            // Extreme Greed zone (75-100)
            { type: 'rect', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: 75, y1: 100, fillcolor: 'rgba(34, 197, 94, 0.1)', line: { width: 0 } },
            // Neutral line
            { type: 'line', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: 50, y1: 50, line: { color: '#6b7280', dash: 'dash', width: 1 } }
        ],
        hovermode: 'x unified'
    }, { responsive: true });
}

// ============================================================================
// BACKTEST SENTIMENT HELPERS
// ============================================================================

async function fetchBacktestSentiment(days) {
    try {
        const response = await fetch(`/api/sentiment/history?days=${days}`);
        const result = await response.json();

        if (result.success && result.stats) {
            const stats = result.stats;

            // Market Mood (Avg)
            document.getElementById('bt-market-avg').textContent = Math.round(stats.average);

            // Classification based on avg
            let mood = "Neutral";
            let moodClass = "neutral";
            if (stats.average >= 55) { mood = "Greed"; moodClass = "greed"; }
            if (stats.average >= 75) { mood = "Extreme Greed"; moodClass = "greed"; }
            if (stats.average <= 45) { mood = "Fear"; moodClass = "fear"; }
            if (stats.average <= 25) { mood = "Extreme Fear"; moodClass = "fear"; }

            const moodEl = document.getElementById('bt-market-mood');
            moodEl.textContent = mood;
            moodEl.className = `metric-value ${moodClass}`;

            document.getElementById('bt-fear-days').textContent = stats.days_in_fear;
            document.getElementById('bt-greed-days').textContent = stats.days_in_greed;
        }
    } catch (e) {
        console.error("Failed to fetch backtest sentiment:", e);
    }
}

function displayAssetSentiment(assetSentiment) {
    if (!assetSentiment || !assetSentiment.current) return;

    const current = assetSentiment.current;

    // Asset RSI
    const rsiEl = document.getElementById('bt-asset-rsi');
    rsiEl.textContent = current.rsi.toFixed(1);
    // Color based on RSI zone
    if (current.rsi > 70) rsiEl.className = 'metric-value negative'; // Overbought warning
    else if (current.rsi < 30) rsiEl.className = 'metric-value positive'; // Oversold opportunity
    else rsiEl.className = 'metric-value';

    // Momentum
    const momEl = document.getElementById('bt-asset-mom');
    momEl.textContent = current.momentum.toFixed(2);
    momEl.className = `metric-value ${current.momentum >= 0 ? 'positive' : 'negative'}`;
}

// ============================================================================
// INIT
// ============================================================================

document.addEventListener('DOMContentLoaded', () => {
    initEquityChart();
    initSymbolSearch();
    updateParams();
    checkHealth();
    loadSymbols();
    loadTradingPairs();

    // Connect WebSocket for live updates
    connectWebSocket();

    document.getElementById('fetch-form').addEventListener('submit', fetchData);
    document.getElementById('backtest-form').addEventListener('submit', runBacktest);

    // Control panel event listeners
    const startBtn = document.getElementById('btn-start');
    const stopBtn = document.getElementById('btn-stop');
    const panicBtn = document.getElementById('btn-panic');

    if (startBtn) startBtn.addEventListener('click', startTrading);
    if (stopBtn) stopBtn.addEventListener('click', stopTrading);
    if (panicBtn) panicBtn.addEventListener('click', panicClose);
});

