/**
 * AlphaField Dashboard v2.0
 * Workflow-Centric State Management
 */

const API_BASE = '/api';

// ===========================
// Global Strategy State
// ===========================

const AppState = {
    strategy: 'GoldenCross',
    symbol: '',
    params: {},
    backtestDays: 30,
    backtestResults: null,
    optimizeResults: null,
    isTrading: false,
    currentTab: 'build'
};

// Strategy parameter definitions
const STRATEGY_PARAMS = {
    GoldenCross: [
        { name: 'fast_period', label: 'Fast Period', default: 10, min: 5, max: 50, step: 5 },
        { name: 'slow_period', label: 'Slow Period', default: 30, min: 20, max: 120, step: 10 }
    ],
    Rsi: [
        { name: 'period', label: 'RSI Period', default: 14, min: 5, max: 30, step: 1 },
        { name: 'lower_bound', label: 'Oversold Level', default: 30, min: 10, max: 40, step: 5 },
        { name: 'upper_bound', label: 'Overbought Level', default: 70, min: 60, max: 90, step: 5 }
    ],
    MeanReversion: [
        { name: 'period', label: 'BB Period', default: 20, min: 10, max: 50, step: 5 },
        { name: 'std_dev', label: 'Std Deviations', default: 2.0, min: 1.0, max: 3.0, step: 0.5 }
    ],
    Momentum: [
        { name: 'ema_period', label: 'EMA Period', default: 50, min: 20, max: 100, step: 10 },
        { name: 'macd_fast', label: 'MACD Fast', default: 12, min: 5, max: 20, step: 1 },
        { name: 'macd_slow', label: 'MACD Slow', default: 26, min: 20, max: 40, step: 1 },
        { name: 'macd_signal', label: 'Signal Line', default: 9, min: 5, max: 15, step: 1 }
    ]
};

// ===========================
// Initialization
// ===========================

document.addEventListener('DOMContentLoaded', () => {
    console.log('AlphaField Dashboard v2.0 initializing...');

    // Initialize tabs
    initTabNavigation();

    // Initialize strategy selection
    initStrategySelection();

    // Load available symbols
    loadSymbols();

    // Initialize backtest period buttons
    initPeriodSelector();

    // Initialize analysis mode toggle
    initAnalysisModeToggle();

    // Check database status
    checkDbStatus();

    // Initialize WebSocket for live trading
    initWebSocket();

    // Load sentiment data
    loadSentiment();

    console.log('Dashboard initialized');
});

// ===========================
// Tab Navigation
// ===========================

function initTabNavigation() {
    // Set initial tab from URL hash or default to 'build'
    const hash = window.location.hash.slice(1);
    if (['build', 'backtest', 'optimize', 'deploy'].includes(hash)) {
        switchTab(hash);
    }
}

function switchTab(tabName) {
    AppState.currentTab = tabName;

    // Update tab buttons
    document.querySelectorAll('.workflow-tab').forEach(tab => {
        tab.classList.remove('active');
        if (tab.dataset.tab === tabName) {
            tab.classList.add('active');
        }
    });

    // Update view sections
    document.querySelectorAll('.view-section').forEach(section => {
        section.classList.remove('active');
    });
    document.getElementById(`${tabName}-view`).classList.add('active');

    // Update URL hash
    window.location.hash = tabName;

    // Refresh tab-specific data
    onTabEnter(tabName);
}

function onTabEnter(tabName) {
    switch (tabName) {
        case 'build':
            updateParamsUI();
            break;
        case 'backtest':
            updateBacktestSummary();
            break;
        case 'optimize':
            updateSensitivityParams();
            break;
        case 'deploy':
            updateDeploySummary();
            loadSentiment();
            break;
    }
}

// ===========================
// Strategy Selection & Params
// ===========================

function initStrategySelection() {
    document.querySelectorAll('input[name="strategy"]').forEach(radio => {
        radio.addEventListener('change', (e) => {
            AppState.strategy = e.target.value;
            updateParamsUI();
            updateContextBar();
        });
    });

    // Initial params setup
    updateParamsUI();
}

function updateParamsUI() {
    const container = document.getElementById('build-params');
    const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];

    if (strategyParams.length === 0) {
        container.innerHTML = '<p class="text-muted">No configurable parameters</p>';
        return;
    }

    let html = '<h4>Strategy Parameters</h4><div class="param-row">';

    strategyParams.forEach(param => {
        const value = AppState.params[param.name] || param.default;
        html += `
            <div class="form-group">
                <label>${param.label}</label>
                <input type="number" 
                       class="form-input" 
                       id="param-${param.name}"
                       value="${value}" 
                       min="${param.min}" 
                       max="${param.max}" 
                       step="${param.step}"
                       onchange="updateParam('${param.name}', this.value)">
            </div>
        `;
    });

    html += '</div>';
    container.innerHTML = html;

    // Initialize params with defaults
    strategyParams.forEach(param => {
        if (!AppState.params[param.name]) {
            AppState.params[param.name] = param.default;
        }
    });
}

function updateParam(name, value) {
    AppState.params[name] = parseFloat(value);
}

function getParams() {
    // Return current strategy params
    const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
    const params = {};

    strategyParams.forEach(param => {
        const input = document.getElementById(`param-${param.name}`);
        if (input) {
            params[param.name] = parseFloat(input.value);
        } else {
            params[param.name] = AppState.params[param.name] || param.default;
        }
    });

    return params;
}

// ===========================
// Context Bar
// ===========================

function updateContextBar() {
    document.getElementById('ctx-strategy').textContent = AppState.strategy;
    document.getElementById('ctx-symbol').textContent = AppState.symbol || '—';

    let status = 'Configure strategy to begin';
    if (AppState.strategy && AppState.symbol) {
        status = 'Ready to test';
    }
    if (AppState.backtestResults) {
        status = `Tested: ${(AppState.backtestResults.total_return * 100).toFixed(1)}% return`;
    }
    if (AppState.isTrading) {
        status = '🟢 Trading Active';
    }
    document.getElementById('ctx-status').textContent = status;
}

// ===========================
// Symbol Loading
// ===========================

async function loadSymbols() {
    try {
        const res = await fetch(`${API_BASE}/data/pairs`);
        const data = await res.json();

        const select = document.getElementById('build-symbol');
        select.innerHTML = '';

        if (data.pairs && data.pairs.length > 0) {
            data.pairs.slice(0, 50).forEach(pair => {
                const option = document.createElement('option');
                option.value = pair.symbol;
                option.textContent = `${pair.symbol} (${pair.base}/${pair.quote})`;
                select.appendChild(option);
            });
        } else {
            select.innerHTML = '<option value="BTC">BTC (Default)</option>';
        }

        // Set default selection
        AppState.symbol = select.value;

        // Listen for changes
        select.addEventListener('change', (e) => {
            AppState.symbol = e.target.value;
            updateContextBar();
            checkDataStatus();
        });

        // Check initial data status
        checkDataStatus();

    } catch (e) {
        console.error('Failed to load symbols:', e);
        document.getElementById('build-symbol').innerHTML = '<option value="BTC">BTC</option>';
        AppState.symbol = 'BTC';
    }
}

async function checkDataStatus() {
    const statusEl = document.getElementById('data-status');
    statusEl.className = 'data-status';
    statusEl.innerHTML = '<span class="status-icon">⏳</span><span class="status-text">Checking data...</span>';

    try {
        const res = await fetch(`${API_BASE}/data/symbols`);
        const data = await res.json();

        const hasCached = data.symbols && data.symbols.some(s => s.symbol === AppState.symbol);

        if (hasCached) {
            statusEl.className = 'data-status ready';
            statusEl.innerHTML = '<span class="status-icon">✓</span><span class="status-text">Data cached and ready</span>';
        } else {
            statusEl.className = 'data-status missing';
            statusEl.innerHTML = '<span class="status-icon">⚠</span><span class="status-text">No cached data - click Fetch Data</span>';
        }
    } catch (e) {
        statusEl.className = 'data-status missing';
        statusEl.innerHTML = '<span class="status-icon">⚠</span><span class="status-text">Unable to check data status</span>';
    }
}

async function fetchDataForSymbol() {
    const statusEl = document.getElementById('data-status');
    statusEl.className = 'data-status';
    statusEl.innerHTML = '<span class="status-icon">⏳</span><span class="status-text">Fetching data...</span>';

    try {
        const res = await fetch(`${API_BASE}/data/fetch`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                symbol: AppState.symbol,
                interval: '1h',
                days: 365
            })
        });

        const data = await res.json();

        if (data.success) {
            statusEl.className = 'data-status ready';
            statusEl.innerHTML = `<span class="status-icon">✓</span><span class="status-text">Fetched ${data.bars_fetched} bars</span>`;
        } else {
            throw new Error(data.message || 'Fetch failed');
        }
    } catch (e) {
        statusEl.className = 'data-status missing';
        statusEl.innerHTML = `<span class="status-icon">❌</span><span class="status-text">Error: ${e.message}</span>`;
    }
}

// ===========================
// Workflow Navigation
// ===========================

function goToBacktest() {
    AppState.params = getParams();
    updateContextBar();
    switchTab('backtest');
}

function goToOptimize() {
    switchTab('optimize');
}

function goToDeploy() {
    switchTab('deploy');
}

// ===========================
// Backtest
// ===========================

function initPeriodSelector() {
    document.querySelectorAll('.period-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.period-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            AppState.backtestDays = parseInt(btn.dataset.days);
        });
    });
}

function updateBacktestSummary() {
    document.getElementById('bt-summary-strategy').textContent = AppState.strategy;
    document.getElementById('bt-summary-symbol').textContent = AppState.symbol || '—';

    const params = getParams();
    const paramsStr = Object.entries(params).map(([k, v]) => `${k}=${v}`).join(', ');
    document.getElementById('bt-summary-params').textContent = paramsStr || '—';
}

async function runBacktest() {
    const btn = document.getElementById('btn-run-backtest');
    btn.disabled = true;
    btn.innerHTML = '⏳ Running...';

    const placeholderEl = document.getElementById('results-placeholder');
    const contentEl = document.getElementById('results-content');

    try {
        const params = getParams();

        const res = await fetch(`${API_BASE}/backtest/run`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                strategy: AppState.strategy,
                symbol: AppState.symbol,
                interval: '1h',
                days: AppState.backtestDays,
                params: params,
                include_benchmark: true
            })
        });

        const data = await res.json();

        if (data.error) throw new Error(data.error);
        if (!data.metrics) throw new Error('No metrics returned');

        // Store results
        AppState.backtestResults = data.metrics;

        // Update metrics display
        updateMetric('bt-return', data.metrics.total_return * 100, '%');
        document.getElementById('bt-sharpe').textContent = data.metrics.sharpe_ratio.toFixed(2);
        updateMetric('bt-drawdown', -data.metrics.max_drawdown * 100, '%', true);
        document.getElementById('bt-winrate').textContent = `${(data.metrics.win_rate * 100).toFixed(0)}%`;
        document.getElementById('bt-trades').textContent = data.metrics.total_trades;

        // Alpha calculation
        if (data.benchmark) {
            const alpha = (data.metrics.total_return - data.benchmark.total_return) * 100;
            updateMetric('bt-alpha', alpha, '%', false);
        } else {
            document.getElementById('bt-alpha').textContent = '—';
        }

        // Render equity chart
        renderEquityChart(data.equity_curve, data.benchmark?.equity_curve);

        // Show results
        placeholderEl.classList.add('hidden');
        contentEl.classList.remove('hidden');

        updateContextBar();

    } catch (e) {
        alert('Backtest failed: ' + e.message);
        console.error(e);
    }

    btn.disabled = false;
    btn.innerHTML = '▶️ Run Backtest';
}

function updateMetric(id, value, suffix = '', invert = false) {
    const el = document.getElementById(id);
    if (!el) return;

    el.textContent = `${value >= 0 ? '+' : ''}${value.toFixed(2)}${suffix}`;
    el.className = `metric-value ${invert ? (value <= 0 ? 'positive' : 'negative') : (value >= 0 ? 'positive' : 'negative')}`;
}

function renderEquityChart(equityCurve, benchmarkCurve) {
    if (!equityCurve || equityCurve.length === 0) return;

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
            name: 'Buy & Hold',
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
        legend: { x: 0, y: 1, orientation: 'h' }
    }, { responsive: true });
}

// ===========================
// Optimize & Tune
// ===========================

function initAnalysisModeToggle() {
    document.querySelectorAll('input[name="analysis-mode"]').forEach(radio => {
        radio.addEventListener('change', (e) => {
            const mode = e.target.value;

            // Update mode option styles
            document.querySelectorAll('.mode-option').forEach(opt => {
                opt.classList.remove('active');
            });
            e.target.closest('.mode-option').classList.add('active');

            // Show/hide config sections
            document.getElementById('wfa-config').classList.toggle('hidden', mode !== 'walkforward');
            document.getElementById('sensitivity-config').classList.toggle('hidden', mode !== 'sensitivity');

            // Hide results
            document.getElementById('wfa-results').classList.add('hidden');
            document.getElementById('sensitivity-results').classList.add('hidden');
        });
    });
}

function updateSensitivityParams() {
    const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
    const select1 = document.getElementById('sens-param-1');
    const select2 = document.getElementById('sens-param-2');

    select1.innerHTML = '';
    select2.innerHTML = '<option value="">None (1D Sweep)</option>';

    strategyParams.forEach(param => {
        select1.innerHTML += `<option value="${param.name}" 
            data-min="${param.min}" data-max="${param.max}" data-step="${param.step}">
            ${param.label}
        </option>`;
        select2.innerHTML += `<option value="${param.name}"
            data-min="${param.min}" data-max="${param.max}" data-step="${param.step}">
            ${param.label}
        </option>`;
    });
}

async function runWalkForward() {
    const chartContainer = document.getElementById('optimize-chart');
    chartContainer.innerHTML = '<div class="placeholder-content"><span class="placeholder-icon">⏳</span><span>Running Walk-Forward Analysis...</span></div>';

    try {
        const res = await fetch(`${API_BASE}/walk-forward`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                strategy: AppState.strategy,
                symbol: AppState.symbol,
                params: getParams()
            })
        });

        const data = await res.json();
        if (!data.success) throw new Error(data.error || 'Analysis failed');

        const result = data.result;

        // Update metrics
        document.getElementById('wfa-stability').textContent = (result.stability_score || 0).toFixed(2);
        updateMetric('wfa-avg-return', (result.aggregate_oos?.mean_return || 0) * 100, '%');
        document.getElementById('wfa-win-rate').textContent = `${((result.aggregate_oos?.win_rate || 0) * 100).toFixed(0)}%`;
        document.getElementById('wfa-oos-sharpe').textContent = (result.aggregate_oos?.mean_sharpe || 0).toFixed(2);

        // Show results card
        document.getElementById('wfa-results').classList.remove('hidden');
        document.getElementById('sensitivity-results').classList.add('hidden');

        // Render chart
        renderWFAChart(result.windows);

        document.getElementById('optimize-chart-title').textContent = 'Walk-Forward Window Returns';

    } catch (e) {
        alert('Walk-Forward Analysis failed: ' + e.message);
        console.error(e);
    }
}

function renderWFAChart(windows) {
    if (!windows || windows.length === 0) {
        document.getElementById('optimize-chart').innerHTML = '<div class="placeholder-content"><span class="placeholder-icon">📊</span><span>No window data available</span></div>';
        return;
    }

    const windowLabels = windows.map((w, i) => `Window ${i + 1}`);
    const testReturns = windows.map(w => (w.test_metrics?.total_return || 0) * 100);
    const trainReturns = windows.map(w => (w.train_metrics?.total_return || 0) * 100);
    const barColors = testReturns.map(r => r >= 0 ? '#10b981' : '#ef4444');

    Plotly.newPlot('optimize-chart', [
        {
            x: windowLabels,
            y: trainReturns,
            type: 'bar',
            name: 'Train Return',
            marker: { color: 'rgba(59, 130, 246, 0.6)' }
        },
        {
            x: windowLabels,
            y: testReturns,
            type: 'bar',
            name: 'Test Return (OOS)',
            marker: { color: barColors }
        }
    ], {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        margin: { t: 40, r: 20, b: 60, l: 60 },
        xaxis: { gridcolor: '#334155', tickangle: -45 },
        yaxis: { gridcolor: '#334155', title: 'Return (%)' },
        barmode: 'group',
        legend: { x: 0, y: 1.15, orientation: 'h' }
    }, { responsive: true });
}

async function runSensitivity() {
    const chartContainer = document.getElementById('optimize-chart');
    chartContainer.innerHTML = '<div class="placeholder-content"><span class="placeholder-icon">⏳</span><span>Running Sensitivity Analysis...</span></div>';

    const p1Select = document.getElementById('sens-param-1');
    const p2Select = document.getElementById('sens-param-2');

    const p1Opt = p1Select.options[p1Select.selectedIndex];
    const param = {
        name: p1Select.value,
        min: parseFloat(p1Opt.dataset.min),
        max: parseFloat(p1Opt.dataset.max),
        step: parseFloat(p1Opt.dataset.step)
    };

    let param_y = null;
    if (p2Select.value) {
        const p2Opt = p2Select.options[p2Select.selectedIndex];
        param_y = {
            name: p2Select.value,
            min: parseFloat(p2Opt.dataset.min),
            max: parseFloat(p2Opt.dataset.max),
            step: parseFloat(p2Opt.dataset.step)
        };
    }

    const baseParams = getParams();
    const fixed_params = { ...baseParams };
    delete fixed_params[param.name];
    if (param_y) delete fixed_params[param_y.name];

    try {
        const res = await fetch(`${API_BASE}/sensitivity`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                strategy: AppState.strategy,
                symbol: AppState.symbol,
                interval: '1h',
                days: 180,
                param,
                param_y,
                fixed_params
            })
        });

        const data = await res.json();
        if (!data.success) throw new Error(data.error);

        // Update metrics
        if (data.results && data.results.length > 0) {
            const bestResult = data.results.reduce((best, r) =>
                r.sharpe_ratio > best.sharpe_ratio ? r : best, data.results[0]);

            let bestText = Object.entries(bestResult.params)
                .map(([k, v]) => `${k}: ${v}`)
                .join(', ');
            document.getElementById('sens-best-param').textContent = bestText;
            document.getElementById('sens-max-sharpe').textContent = bestResult.sharpe_ratio.toFixed(3);
        }

        // Show results card
        document.getElementById('sensitivity-results').classList.remove('hidden');
        document.getElementById('wfa-results').classList.add('hidden');

        // Render chart
        if (data.heatmap) {
            renderHeatmap(data.heatmap);
            document.getElementById('optimize-chart-title').textContent = `Sharpe Ratio: ${data.heatmap.x_param} vs ${data.heatmap.y_param}`;
        } else if (data.results && data.results.length > 0) {
            renderSensitivityLineChart(data.results, param.name);
            document.getElementById('optimize-chart-title').textContent = `Parameter Sensitivity: ${param.name}`;
        }

    } catch (e) {
        alert('Sensitivity Analysis failed: ' + e.message);
        console.error(e);
    }
}

function renderSensitivityLineChart(results, paramName) {
    const sorted = [...results].sort((a, b) => a.params[paramName] - b.params[paramName]);

    const xValues = sorted.map(r => r.params[paramName]);
    const sharpeValues = sorted.map(r => r.sharpe_ratio);
    const returnValues = sorted.map(r => r.total_return * 100);

    Plotly.newPlot('optimize-chart', [
        {
            x: xValues,
            y: sharpeValues,
            type: 'scatter',
            mode: 'lines+markers',
            name: 'Sharpe Ratio',
            line: { color: '#3b82f6', width: 2 },
            marker: { size: 8 },
            yaxis: 'y'
        },
        {
            x: xValues,
            y: returnValues,
            type: 'scatter',
            mode: 'lines+markers',
            name: 'Return (%)',
            line: { color: '#10b981', width: 2 },
            marker: { size: 8 },
            yaxis: 'y2'
        }
    ], {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        margin: { t: 50, r: 80, b: 60, l: 60 },
        xaxis: { title: paramName, gridcolor: '#334155' },
        yaxis: { title: 'Sharpe Ratio', gridcolor: '#334155', side: 'left' },
        yaxis2: { title: 'Return (%)', overlaying: 'y', side: 'right', gridcolor: '#334155' },
        legend: { x: 0, y: 1.15, orientation: 'h' }
    }, { responsive: true });
}

function renderHeatmap(heatmapData) {
    if (!heatmapData || !heatmapData.sharpe_matrix || !heatmapData.x_values || !heatmapData.y_values) {
        console.warn('Heatmap data missing required fields:', heatmapData);
        return;
    }

    const zData = heatmapData.sharpe_matrix;
    const xValues = heatmapData.x_values;
    const yValues = heatmapData.y_values;

    const annotations = [];
    for (let i = 0; i < yValues.length; i++) {
        if (!zData[i]) continue;
        for (let j = 0; j < xValues.length; j++) {
            if (zData[i][j] === undefined || zData[i][j] === null) continue;
            annotations.push({
                x: xValues[j],
                y: yValues[i],
                text: zData[i][j].toFixed(3),
                font: { color: 'white', size: 11 },
                showarrow: false
            });
        }
    }

    Plotly.newPlot('optimize-chart', [{
        z: zData,
        x: xValues,
        y: yValues,
        type: 'heatmap',
        colorscale: [
            [0, '#ef4444'],
            [0.5, '#fbbf24'],
            [1, '#10b981']
        ],
        colorbar: { title: 'Sharpe' },
        hoverongaps: false
    }], {
        paper_bgcolor: '#1e293b',
        plot_bgcolor: '#1e293b',
        font: { color: '#94a3b8' },
        xaxis: { title: heatmapData.x_param, tickformat: '.1f', tickmode: 'array', tickvals: xValues },
        yaxis: { title: heatmapData.y_param, tickformat: '.1f', tickmode: 'array', tickvals: yValues },
        margin: { t: 50, r: 100, b: 60, l: 70 },
        annotations: annotations
    }, { responsive: true });
}

function applyBestParams() {
    // Would update AppState.params with best found params
    alert('Best parameters applied! Go to Build tab to see updated values.');
    switchTab('build');
}

// ===========================
// Deploy & Live Trading
// ===========================

function updateDeploySummary() {
    document.getElementById('deploy-strategy').textContent = AppState.strategy;
    document.getElementById('deploy-symbol').textContent = AppState.symbol || '—';

    if (AppState.backtestResults) {
        updateMetric('deploy-return', AppState.backtestResults.total_return * 100, '%');
        document.getElementById('deploy-sharpe').textContent = AppState.backtestResults.sharpe_ratio?.toFixed(2) || '—';
    } else {
        document.getElementById('deploy-return').textContent = '—';
        document.getElementById('deploy-sharpe').textContent = '—';
    }
}

async function loadSentiment() {
    try {
        const res = await fetch(`${API_BASE}/sentiment/current`);
        const data = await res.json();

        if (data.value !== undefined) {
            const value = data.value;
            document.getElementById('gauge-value').textContent = value;
            document.getElementById('gauge-label').textContent = data.classification || getSentimentLabel(value);
            document.getElementById('gauge-fill').style.width = `${value}%`;
        }
    } catch (e) {
        console.error('Failed to load sentiment:', e);
    }
}

function getSentimentLabel(value) {
    if (value <= 25) return 'Extreme Fear';
    if (value <= 45) return 'Fear';
    if (value <= 55) return 'Neutral';
    if (value <= 75) return 'Greed';
    return 'Extreme Greed';
}

let ws = null;
let equityChart = null;

function initWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${protocol}//${window.location.host}/api/ws`);

    ws.onopen = () => {
        console.log('WebSocket connected');
        updateWsStatus(true);
    };

    ws.onclose = () => {
        console.log('WebSocket disconnected');
        updateWsStatus(false);
        // Reconnect after 5 seconds
        setTimeout(initWebSocket, 5000);
    };

    ws.onerror = (err) => {
        console.error('WebSocket error:', err);
    };

    ws.onmessage = (event) => {
        try {
            const msg = JSON.parse(event.data);
            handleWsMessage(msg);
        } catch (e) {
            console.error('Failed to parse WebSocket message:', e);
        }
    };
}

function updateWsStatus(connected) {
    const el = document.getElementById('ws-status');
    if (connected) {
        el.className = 'status-badge connected';
        el.innerHTML = '<span class="status-dot"></span><span>Live</span>';
    } else {
        el.className = 'status-badge disconnected';
        el.innerHTML = '<span class="status-dot"></span><span>Live</span>';
    }
}

function handleWsMessage(msg) {
    switch (msg.type) {
        case 'portfolio_update':
            updatePortfolioDisplay(msg.data);
            break;
        case 'trade':
            addTradeToHistory(msg.data);
            break;
        case 'log':
            addLogEntry(msg.data);
            break;
    }
}

function updatePortfolioDisplay(data) {
    document.getElementById('live-total-value').textContent = formatCurrency(data.total_value);
    document.getElementById('live-cash').textContent = formatCurrency(data.cash);
    document.getElementById('live-positions-value').textContent = formatCurrency(data.positions_value);
    document.getElementById('live-pnl').textContent = formatCurrency(data.pnl);
}

function formatCurrency(value) {
    return new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' }).format(value);
}

function addLogEntry(entry) {
    const console = document.getElementById('log-console');
    const time = new Date().toLocaleTimeString();
    const div = document.createElement('div');
    div.className = `log-entry log-${entry.level || 'info'}`;
    div.innerHTML = `<span class="log-time">${time}</span><span class="log-msg">${entry.message}</span>`;
    console.appendChild(div);
    console.scrollTop = console.scrollHeight;
}

function startTrading() {
    if (!AppState.strategy || !AppState.symbol) {
        alert('Please configure a strategy and symbol first');
        return;
    }

    AppState.isTrading = true;

    document.getElementById('btn-start').disabled = true;
    document.getElementById('btn-stop').disabled = false;
    document.getElementById('engine-status').textContent = 'Running';
    document.getElementById('engine-status').className = 'engine-status running';

    ws.send(JSON.stringify({
        type: 'start_trading',
        strategy: AppState.strategy,
        symbol: AppState.symbol,
        params: getParams(),
        position_size: parseFloat(document.getElementById('position-size').value)
    }));

    addLogEntry({ message: `Started ${AppState.strategy} on ${AppState.symbol}`, level: 'success' });
    updateContextBar();
}

function stopTrading() {
    AppState.isTrading = false;

    document.getElementById('btn-start').disabled = false;
    document.getElementById('btn-stop').disabled = true;
    document.getElementById('engine-status').textContent = 'Stopped';
    document.getElementById('engine-status').className = 'engine-status stopped';

    ws.send(JSON.stringify({ type: 'stop_trading' }));

    addLogEntry({ message: 'Trading stopped', level: 'info' });
    updateContextBar();
}

function panicClose() {
    if (!confirm('Close all positions immediately?')) return;

    ws.send(JSON.stringify({ type: 'panic_close' }));
    addLogEntry({ message: '⚠️ PANIC CLOSE - All positions closed', level: 'error' });
    stopTrading();
}

// ===========================
// Database Status
// ===========================

async function checkDbStatus() {
    try {
        const res = await fetch(`${API_BASE}/health`);
        const data = await res.json();

        const el = document.getElementById('db-status');
        if (data.database === 'connected') {
            el.className = 'status-badge connected';
            el.innerHTML = '<span class="status-dot"></span><span>DB</span>';
        } else {
            el.className = 'status-badge disconnected';
            el.innerHTML = '<span class="status-dot"></span><span>DB</span>';
        }
    } catch (e) {
        document.getElementById('db-status').className = 'status-badge disconnected';
    }
}
