const API_BASE = '/api';

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
    if (tabName === 'data') loadSymbols();
    if (tabName === 'backtest') populateSymbolDropdown();
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

let btChart;

function initBacktestChart() {
    const ctx = document.getElementById('backtestChart').getContext('2d');
    btChart = new Chart(ctx, {
        type: 'line',
        data: { labels: [], datasets: [{ label: 'Equity', data: [], borderColor: '#10b981', backgroundColor: 'rgba(16, 185, 129, 0.1)', tension: 0.1, fill: true }] },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: { legend: { display: false } },
            scales: {
                y: { grid: { color: '#334155' }, ticks: { color: '#94a3b8' } },
                x: { grid: { display: false }, ticks: { color: '#94a3b8', maxTicksLimit: 10 } }
            }
        }
    });
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

        const params = {};
        document.getElementById('strategy-params').querySelectorAll('input').forEach(input => {
            params[input.name] = parseFloat(input.value);
        });

        const res = await fetch(`${API_BASE}/backtest/run`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ strategy, symbol, interval: '1h', days, params })
        });

        const data = await res.json();

        // Update metrics
        document.getElementById('bt-return').textContent = formatPercent(data.metrics.total_return * 100);
        document.getElementById('bt-return').className = `metric-value ${colorClass(data.metrics.total_return)}`;
        document.getElementById('bt-cagr').textContent = formatPercent(data.metrics.cagr * 100);
        document.getElementById('bt-sharpe').textContent = data.metrics.sharpe_ratio.toFixed(2);
        document.getElementById('bt-drawdown').textContent = formatPercent(data.metrics.max_drawdown * 100);
        document.getElementById('bt-volatility').textContent = formatPercent(data.metrics.volatility * 100);

        // Update chart
        const labels = data.equity_curve.map(p => new Date(p[0]).toLocaleDateString());
        const values = data.equity_curve.map(p => p[1]);
        btChart.data.labels = labels;
        btChart.data.datasets[0].data = values;
        btChart.update();

    } catch (e) {
        alert('Backtest failed: ' + e.message);
    }

    btn.disabled = false;
    btn.innerHTML = '<span>▶️</span> Run Backtest';
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
// LIVE VIEW (Mock)
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
// INIT
// ============================================================================

document.addEventListener('DOMContentLoaded', () => {
    initBacktestChart();
    initEquityChart();
    updateParams();
    checkHealth();
    loadSymbols();

    document.getElementById('fetch-form').addEventListener('submit', fetchData);
    document.getElementById('backtest-form').addEventListener('submit', runBacktest);
});
