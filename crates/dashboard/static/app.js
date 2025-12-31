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
    backtestDays: 90,
    backtestInterval: '1h',
    backtestResults: null,
    optimizeResults: null,
    isTrading: false,
    currentTab: 'build'
};

// Strategy parameter definitions
const STRATEGY_PARAMS = {
    GoldenCross: [
        { name: 'fast_period', label: 'Fast Period', default: 10, min: 5, max: 50, step: 5 },
        { name: 'slow_period', label: 'Slow Period', default: 30, min: 20, max: 120, step: 10 },
        { name: 'take_profit', label: 'Take Profit (%)', default: 5.0, min: 1.0, max: 50.0, step: 1.0 },
        { name: 'stop_loss', label: 'Stop Loss (%)', default: 5.0, min: 1.0, max: 20.0, step: 0.5 }
    ],
    Rsi: [
        { name: 'period', label: 'RSI Period', default: 14, min: 5, max: 30, step: 1 },
        { name: 'lower_bound', label: 'Oversold Level', default: 30, min: 10, max: 40, step: 5 },
        { name: 'upper_bound', label: 'Overbought Level', default: 70, min: 60, max: 90, step: 5 },
        { name: 'take_profit', label: 'Take Profit (%)', default: 3.0, min: 1.0, max: 50.0, step: 0.5 },
        { name: 'stop_loss', label: 'Stop Loss (%)', default: 5.0, min: 1.0, max: 20.0, step: 0.5 }
    ],
    MeanReversion: [
        { name: 'period', label: 'BB Period', default: 20, min: 10, max: 50, step: 5 },
        { name: 'std_dev', label: 'Std Deviations', default: 2.0, min: 1.0, max: 3.0, step: 0.5 },
        { name: 'take_profit', label: 'Take Profit (%)', default: 3.0, min: 1.0, max: 50.0, step: 0.5 },
        { name: 'stop_loss', label: 'Stop Loss (%)', default: 5.0, min: 1.0, max: 20.0, step: 0.5 }
    ],
    Momentum: [
        { name: 'ema_period', label: 'EMA Period', default: 50, min: 20, max: 100, step: 10 },
        { name: 'macd_fast', label: 'MACD Fast', default: 12, min: 5, max: 20, step: 1 },
        { name: 'macd_slow', label: 'MACD Slow', default: 26, min: 20, max: 40, step: 1 },
        { name: 'macd_signal', label: 'Signal Line', default: 9, min: 5, max: 15, step: 1 },
        { name: 'take_profit', label: 'Take Profit (%)', default: 5.0, min: 1.0, max: 50.0, step: 0.5 },
        { name: 'stop_loss', label: 'Stop Loss (%)', default: 5.0, min: 1.0, max: 20.0, step: 0.5 }
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

    // Initialize interval selector
    initIntervalSelector();

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

// ===========================
// Symbol Loading & Custom Select
// ===========================

// State for symbol data
let allSymbols = [];

async function loadSymbols() {
    try {
        const res = await fetch(`${API_BASE}/data/pairs`);
        const data = await res.json();

        // Handle inconsistent API responses (array of strings vs object with pairs array)
        if (Array.isArray(data)) {
            allSymbols = data;
        } else if (data.pairs && Array.isArray(data.pairs)) {
            allSymbols = data.pairs.map(p => typeof p === 'string' ? p : p.symbol);
        } else {
            // Fallback for unexpected structure
            allSymbols = [];
            console.warn('Unexpected symbol data structure:', data);
        }

        // Default popular symbols if we have data
        if (allSymbols.length > 0) {
            initCustomSelect();
            initOptimizeSymbolSelect(); // Also initialize optimize tab selector
        } else {
            document.getElementById('selected-symbol-text').textContent = "No symbols found";
            const optimizeSymbolText = document.getElementById('optimize-selected-symbol-text');
            if (optimizeSymbolText) optimizeSymbolText.textContent = "No symbols found";
        }

    } catch (e) {
        console.error('Failed to load symbols:', e);
        document.getElementById('selected-symbol-text').textContent = "Error loading symbols";
        const optimizeSymbolText = document.getElementById('optimize-selected-symbol-text');
        if (optimizeSymbolText) optimizeSymbolText.textContent = "Error loading symbols";
        // Fallback
        allSymbols = ["BTC", "ETH"];
        initCustomSelect();
        initOptimizeSymbolSelect();
    }
}

function initCustomSelect() {
    const wrapper = document.getElementById('symbol-select-wrapper');
    const trigger = document.getElementById('symbol-trigger');
    const optionsList = document.getElementById('symbol-options-list');
    const searchInput = document.getElementById('symbol-search-input');
    const hiddenInput = document.getElementById('build-symbol');
    const selectedText = document.getElementById('selected-symbol-text');

    // Toggle dropdown
    trigger.addEventListener('click', () => {
        wrapper.classList.toggle('open');
        if (wrapper.classList.contains('open')) {
            searchInput.focus();
        }
    });

    // Close when clicking outside
    document.addEventListener('click', (e) => {
        if (!wrapper.contains(e.target)) {
            wrapper.classList.remove('open');
        }
    });

    // Filter function
    function filterSymbols(query) {
        const q = query.toLowerCase();
        const POPULAR = ['BTC', 'ETH', 'SOL', 'XRP', 'ADA', 'DOGE', 'AVAX', 'DOT', 'LINK', 'MATIC'];

        let filtered = allSymbols.filter(s => s.toLowerCase().includes(q));

        // Sort: Popular first, then alphabetical
        filtered.sort((a, b) => {
            const aPop = POPULAR.indexOf(a);
            const bPop = POPULAR.indexOf(b);

            if (aPop !== -1 && bPop !== -1) return aPop - bPop;
            if (aPop !== -1) return -1;
            if (bPop !== -1) return 1;
            return a.localeCompare(b);
        });

        renderOptions(filtered);
    }

    // Render options to the list
    function renderOptions(symbols) {
        optionsList.innerHTML = '';

        if (symbols.length === 0) {
            optionsList.innerHTML = '<div class="option" style="cursor: default; opacity: 0.5;">No matches found</div>';
            return;
        }

        // Limit rendering for performance if list is huge
        const displaySymbols = symbols.slice(0, 100);

        displaySymbols.forEach(symbol => {
            const div = document.createElement('div');
            div.className = 'option';
            if (symbol === AppState.symbol) div.classList.add('selected');

            div.innerHTML = `
                <span class="option-ticker">${symbol}</span>
                <span class="option-name">USDT</span>
            `;

            div.addEventListener('click', () => {
                selectSymbol(symbol);
            });

            optionsList.appendChild(div);
        });

        if (symbols.length > 100) {
            const more = document.createElement('div');
            more.className = 'option';
            more.style.opacity = '0.5';
            more.style.fontStyle = 'italic';
            more.textContent = `...and ${symbols.length - 100} more`;
            optionsList.appendChild(more);
        }
    }

    // Selection handler
    function selectSymbol(symbol) {
        AppState.symbol = symbol;
        hiddenInput.value = symbol;
        selectedText.textContent = symbol;
        wrapper.classList.remove('open');

        // Update UI state
        updateContextBar();

        // Re-render to update selected styling
        filterSymbols(searchInput.value);
    }

    // Search input handler
    searchInput.addEventListener('input', (e) => {
        filterSymbols(e.target.value);
    });

    // Initial render
    filterSymbols('');

    // Select default or first available
    if (!AppState.symbol && allSymbols.length > 0) {
        // Default to BTC if available, or first item
        const defaultSym = allSymbols.includes('BTC') ? 'BTC' : allSymbols[0];
        selectSymbol(defaultSym);
    } else if (AppState.symbol) {
        selectSymbol(AppState.symbol);
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

function initIntervalSelector() {
    document.querySelectorAll('.interval-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            document.querySelectorAll('.interval-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            AppState.backtestInterval = btn.dataset.interval;
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

    const placeholderEl = document.getElementById('backtest-results-placeholder');
    const contentEl = document.getElementById('backtest-results-content');

    try {
        const params = getParams();

        const res = await fetch(`${API_BASE}/backtest/run`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                strategy: AppState.strategy,
                symbol: AppState.symbol,
                interval: AppState.backtestInterval,
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
        // Fix: Benchmark metrics are nested in data.benchmark.comparison and use benchmark_return
        if (data.benchmark && data.benchmark.comparison && data.benchmark.comparison.benchmark_return !== undefined) {
            const alpha = (data.metrics.total_return - data.benchmark.comparison.benchmark_return) * 100;
            if (isNaN(alpha)) {
                document.getElementById('bt-alpha').textContent = '—';
            } else {
                updateMetric('bt-alpha', alpha, '%', false);
            }
        } else {
            console.warn('Benchmark data missing or invalid structure', data.benchmark);
            document.getElementById('bt-alpha').textContent = '—';
        }

        // Render equity chart with trade markers
        renderEquityChart(data.equity_curve, data.benchmark?.curve, data.trades);

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
// ===========================
// Parameter Optimization
// ===========================

// Store optimization results for visualization
let lastOptimizeResults = null;

async function optimizeParams() {
    const btn = document.getElementById('btn-auto-optimize');
    const runBtn = document.getElementById('btn-run-backtest');
    btn.disabled = true;
    runBtn.disabled = true;
    btn.innerHTML = '⏳ Optimizing...';

    try {
        const res = await fetch(`${API_BASE}/backtest/optimize`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                strategy: AppState.strategy,
                symbol: AppState.symbol,
                interval: AppState.backtestInterval,
                days: AppState.backtestDays
            })
        });

        const data = await res.json();

        if (!data.success) {
            throw new Error(data.error || 'Optimization failed');
        }

        // Store results for later reference
        lastOptimizeResults = data;

        // Apply optimized params to AppState
        AppState.params = data.optimized_params;

        // Update UI inputs with optimized values
        const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
        strategyParams.forEach(param => {
            const value = data.optimized_params[param.name];
            if (value !== undefined) {
                const input = document.getElementById(`param-${param.name}`);
                if (input) {
                    input.value = value;
                }
            }
        });

        // Update backtest summary
        updateBacktestSummary();

        // Show detailed results with visualization
        const returnPct = (data.best_return * 100).toFixed(2);
        const sharpe = data.best_sharpe.toFixed(2);
        const winRate = (data.best_win_rate * 100).toFixed(0);
        const drawdown = (data.best_max_drawdown * 100).toFixed(2);
        const score = data.best_score.toFixed(3);

        alert(`✨ Optimization Complete!

📊 Best Composite Score: ${score}
   - Sharpe Ratio: ${sharpe}
   - Return: ${returnPct}%
   - Win Rate: ${winRate}%
   - Max Drawdown: ${drawdown}%

🔢 Tested ${data.iterations} combinations in ${data.elapsed_ms}ms

Parameters have been applied. 
Click "Run Backtest" to see full results, or go to "Optimize" tab to view all sweep results.`);

        // Navigate to Optimize tab and show results
        switchTab('optimize');
        renderOptimizationSweepChart(data);

    } catch (e) {
        alert('Optimization failed: ' + e.message);
        console.error(e);
    }

    btn.disabled = false;
    runBtn.disabled = false;
    btn.innerHTML = '🎯 Auto-Optimize';
}

// Render optimization sweep results as a scatter chart
function renderOptimizationSweepChart(data) {
    if (!data.sweep_results || data.sweep_results.length === 0) {
        console.warn('No sweep results to visualize');
        return;
    }

    // Sort by score for color gradient
    const results = [...data.sweep_results].sort((a, b) => a.score - b.score);

    // Create scatter plot: Sharpe vs Return, colored by score
    const trace = {
        x: results.map(r => r.sharpe),
        y: results.map(r => r.total_return * 100),
        mode: 'markers',
        type: 'scatter',
        marker: {
            size: 10,
            color: results.map(r => r.score),
            colorscale: 'Viridis',
            colorbar: {
                title: 'Score',
                titleside: 'right'
            },
            line: { color: 'white', width: 1 }
        },
        text: results.map(r => {
            const params = Object.entries(r.params).map(([k, v]) => `${k}: ${v}`).join('<br>');
            return `Score: ${r.score.toFixed(3)}<br>Sharpe: ${r.sharpe.toFixed(2)}<br>Return: ${(r.total_return * 100).toFixed(2)}%<br>Win Rate: ${(r.win_rate * 100).toFixed(0)}%<br>Drawdown: ${(r.max_drawdown * 100).toFixed(2)}%<br>Trades: ${r.total_trades}<br><br>${params}`;
        }),
        hoverinfo: 'text'
    };

    // Mark best result
    const best = results[results.length - 1]; // Highest score
    const bestTrace = {
        x: [best.sharpe],
        y: [best.total_return * 100],
        mode: 'markers',
        type: 'scatter',
        name: 'Best',
        marker: {
            size: 18,
            color: '#10b981',
            symbol: 'star',
            line: { color: 'white', width: 2 }
        },
        hoverinfo: 'skip'
    };

    const layout = {
        paper_bgcolor: 'rgba(0,0,0,0)',
        plot_bgcolor: 'rgba(0,0,0,0)',
        font: { color: '#94a3b8' },
        margin: { t: 40, r: 80, b: 50, l: 60 },
        xaxis: {
            title: 'Sharpe Ratio',
            gridcolor: 'rgba(255, 255, 255, 0.05)',
            zeroline: true,
            zerolinecolor: 'rgba(255, 255, 255, 0.2)'
        },
        yaxis: {
            title: 'Total Return (%)',
            gridcolor: 'rgba(255, 255, 255, 0.05)',
            zeroline: true,
            zerolinecolor: 'rgba(255, 255, 255, 0.2)'
        },
        showlegend: false,
        title: {
            text: `Parameter Sweep Results (${data.iterations} combinations)`,
            font: { color: '#e2e8f0', size: 14 }
        }
    };

    Plotly.newPlot('optimize-chart', [trace, bestTrace], layout, { responsive: true });

    // Update title
    document.getElementById('optimize-chart-title').textContent = 'Parameter Sweep Analysis';

    // Show sensitivity results block
    document.getElementById('optimize-results-placeholder').classList.add('hidden');
    document.getElementById('wfa-results').classList.add('hidden');
    document.getElementById('sensitivity-results').classList.remove('hidden');

    // Update metrics - format params nicely
    const paramsText = Object.entries(data.optimized_params)
        .map(([k, v]) => `${k}=${Number(v).toFixed(1)}`)
        .join(', ');
    document.getElementById('sens-best-param').textContent = paramsText;
    document.getElementById('sens-max-sharpe').textContent = data.best_sharpe.toFixed(2);

}

// Store best params from last optimization
function applyBestParams() {
    if (lastOptimizeResults && lastOptimizeResults.optimized_params) {
        AppState.params = lastOptimizeResults.optimized_params;
        updateParamsUI();
        updateBacktestSummary();
        switchTab('backtest');
    } else {
        alert('No optimization results available. Run Auto-Optimize first.');
    }
}



function updateMetric(id, value, suffix = '', invert = false) {
    const el = document.getElementById(id);
    if (!el) return;

    // Handle NaN or undefined values
    if (isNaN(value) || value === undefined || value === null) {
        el.textContent = '—';
        el.className = 'metric-value';
        return;
    }

    el.textContent = `${value >= 0 ? '+' : ''}${value.toFixed(2)}${suffix}`;
    el.className = `metric-value ${invert ? (value <= 0 ? 'positive' : 'negative') : (value >= 0 ? 'positive' : 'negative')}`;
}

function renderEquityChart(equityCurve, benchmarkCurve, trades = []) {
    if (!equityCurve || equityCurve.length === 0) return;

    const timestamps = equityCurve.map(p => new Date(p.timestamp));
    const equity = equityCurve.map(p => p.equity);

    const traces = [{
        x: timestamps,
        y: equity,
        type: 'scatter',
        mode: 'lines',
        name: 'Strategy',
        line: { color: '#3b82f6', width: 2 },
        hoverlabel: { bgcolor: '#1e293b' }
    }];

    if (benchmarkCurve && benchmarkCurve.length > 0) {
        traces.push({
            x: benchmarkCurve.map(p => new Date(p.timestamp)),
            y: benchmarkCurve.map(p => p.equity),
            type: 'scatter',
            mode: 'lines',
            name: 'Buy & Hold',
            line: { color: '#6b7280', width: 1, dash: 'dot' },
            hoverlabel: { bgcolor: '#1e293b' }
        });
    }

    // Add trade entry markers (green triangles pointing up)
    if (trades && trades.length > 0) {
        const entryX = [];
        const entryY = [];
        const entryText = [];
        const exitX = [];
        const exitY = [];
        const exitText = [];

        trades.forEach(trade => {
            // Find closest equity point for entry
            const entryTime = new Date(trade.entry_time);
            const exitTime = new Date(trade.exit_time);

            // Find equity value at entry time
            let entryEquity = null;
            for (let i = 0; i < equityCurve.length; i++) {
                if (new Date(equityCurve[i].timestamp) >= entryTime) {
                    entryEquity = equityCurve[i].equity;
                    break;
                }
            }
            if (entryEquity === null && equityCurve.length > 0) {
                entryEquity = equityCurve[0].equity;
            }

            // Find equity value at exit time
            let exitEquity = null;
            for (let i = 0; i < equityCurve.length; i++) {
                if (new Date(equityCurve[i].timestamp) >= exitTime) {
                    exitEquity = equityCurve[i].equity;
                    break;
                }
            }
            if (exitEquity === null && equityCurve.length > 0) {
                exitEquity = equityCurve[equityCurve.length - 1].equity;
            }

            entryX.push(entryTime);
            entryY.push(entryEquity);
            entryText.push(`Entry: $${trade.entry_price?.toFixed(2) || '?'}<br>Qty: ${trade.quantity?.toFixed(4) || '?'}`);

            exitX.push(exitTime);
            exitY.push(exitEquity);
            const pnlColor = (trade.pnl || 0) >= 0 ? '#10b981' : '#ef4444';
            exitText.push(`Exit: $${trade.exit_price?.toFixed(2) || '?'}<br>Reason: ${trade.exit_reason || 'Unknown'}<br>PnL: <span style="color:${pnlColor}">$${trade.pnl?.toFixed(2) || '?'}</span>`);
        });

        // Entry markers
        traces.push({
            x: entryX,
            y: entryY,
            type: 'scatter',
            mode: 'markers',
            name: 'Entry',
            marker: {
                color: '#10b981',
                size: 12,
                symbol: 'triangle-up',
                line: { color: 'white', width: 1 }
            },
            text: entryText,
            hoverinfo: 'text+x',
            hoverlabel: { bgcolor: '#1e293b', bordercolor: '#10b981' }
        });

        // Exit markers
        traces.push({
            x: exitX,
            y: exitY,
            type: 'scatter',
            mode: 'markers',
            name: 'Exit',
            marker: {
                color: '#ef4444',
                size: 12,
                symbol: 'triangle-down',
                line: { color: 'white', width: 1 }
            },
            text: exitText,
            hoverinfo: 'text+x',
            hoverlabel: { bgcolor: '#1e293b', bordercolor: '#ef4444' }
        });
    }

    Plotly.newPlot('equity-chart', traces, {
        paper_bgcolor: 'rgba(0,0,0,0)',
        plot_bgcolor: 'rgba(0,0,0,0)',
        font: { color: '#94a3b8' },
        margin: { t: 20, r: 20, b: 40, l: 60 },
        xaxis: { gridcolor: 'rgba(255, 255, 255, 0.05)' },
        yaxis: { gridcolor: 'rgba(255, 255, 255, 0.05)', title: 'Equity ($)' },
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
            document.getElementById('comprehensive-config').classList.toggle('hidden', mode !== 'comprehensive');
            document.getElementById('wfa-config').classList.toggle('hidden', mode !== 'walkforward');
            document.getElementById('sensitivity-config').classList.toggle('hidden', mode !== 'sensitivity');

            // Hide results and show placeholder
            document.getElementById('comprehensive-results').classList.add('hidden');
            document.getElementById('wfa-results').classList.add('hidden');
            document.getElementById('sensitivity-results').classList.add('hidden');
            document.getElementById('optimize-results-placeholder').classList.remove('hidden');
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
    const resultsPlaceholder = document.getElementById('optimize-results-placeholder');
    const originalContent = chartContainer.innerHTML;
    const originalPlaceholder = resultsPlaceholder.innerHTML;

    // Show loading state with more detail in Chart
    chartContainer.innerHTML = `
        <div class="placeholder-content">
            <span class="placeholder-icon">⏳</span>
            <span>Running Walk-Forward Analysis...</span>
            <span style="font-size: 12px; margin-top: 8px; opacity: 0.7;">Fetching 4 years of data and simulating. This may take 10-20 seconds.</span>
        </div>`;

    // Update Results Placeholder to match
    resultsPlaceholder.innerHTML = `
        <span class="placeholder-icon">⏳</span>
        <span>Calculating Metrics...</span>
    `;

    // Disable button to prevent double-submit
    const btn = document.querySelector('button[onclick="runWalkForward()"]');
    if (btn) btn.disabled = true;

    try {
        // Get symbol from optimize tab
        const optimizeSymbol = document.getElementById("optimize-symbol").value;
        if (!optimizeSymbol) {
            throw new Error("Please select a trading symbol first");
        }
        
        // Update AppState
        AppState.symbol = optimizeSymbol;
        
        const res = await fetch(`${API_BASE}/walk-forward`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                strategy: AppState.strategy,
                symbol: optimizeSymbol,
                interval: '1h',
                params: getParams()
            })
        });

        const data = await res.json();

        if (!data.success) {
            throw new Error(data.error || 'Analysis failed');
        }

        const result = data.result;

        // Update metrics
        document.getElementById('wfa-stability').textContent = (result.stability_score || 0).toFixed(2);
        updateMetric('wfa-avg-return', (result.aggregate_oos?.mean_return || 0) * 100, '%');
        document.getElementById('wfa-win-rate').textContent = `${((result.aggregate_oos?.win_rate || 0) * 100).toFixed(0)}%`;
        document.getElementById('wfa-oos-sharpe').textContent = (result.aggregate_oos?.mean_sharpe || 0).toFixed(2);

        // Show results block
        resultsPlaceholder.classList.add('hidden');
        document.getElementById('wfa-results').classList.remove('hidden');
        document.getElementById('sensitivity-results').classList.add('hidden');

        // Render chart
        renderWFAChart(result.windows);

        document.getElementById('optimize-chart-title').textContent = 'Walk-Forward Window Returns';

    } catch (e) {
        console.error("WFA Error:", e);
        // Show error in the chart container
        chartContainer.innerHTML = `
            <div class="placeholder-content" style="color: #ef4444;">
                <span class="placeholder-icon">⚠️</span>
                <span>Analysis Failed</span>
                <span style="font-size: 14px; margin-top: 8px; max-width: 80%; text-align: center;">${e.message}</span>
                <button class="btn-primary" style="margin-top: 16px;" onclick="runWalkForward()">Try Again</button>
            </div>`;

        // Show error in results placeholder too
        resultsPlaceholder.innerHTML = `
            <span class="placeholder-icon" style="color: #ef4444;">⚠️</span>
            <span style="color: #ef4444;">Analysis Failed</span>
        `;
        resultsPlaceholder.classList.remove('hidden'); // Ensure it's visible if we came from success state
        document.getElementById('wfa-results').classList.add('hidden');

    } finally {
        if (btn) btn.disabled = false;
    }
}

function renderWFAChart(windows) {
    const container = document.getElementById('optimize-chart');
    container.innerHTML = ''; // Clear loading state explicitly

    if (!windows || windows.length === 0) {
        container.innerHTML = '<div class="placeholder-content"><span class="placeholder-icon">📊</span><span>No window data available</span></div>';
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
        paper_bgcolor: 'rgba(0,0,0,0)',
        plot_bgcolor: 'rgba(0,0,0,0)',
        font: { color: '#94a3b8' },
        margin: { t: 40, r: 20, b: 60, l: 60 },
        xaxis: { gridcolor: 'rgba(255, 255, 255, 0.05)', tickangle: -45 },
        yaxis: { gridcolor: 'rgba(255, 255, 255, 0.05)', title: 'Return (%)' },
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
        // Get symbol from optimize tab
        const optimizeSymbol = document.getElementById("optimize-symbol").value;
        if (!optimizeSymbol) {
            throw new Error("Please select a trading symbol first");
        }
        
        // Update AppState
        AppState.symbol = optimizeSymbol;
        
        const res = await fetch(`${API_BASE}/sensitivity`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                strategy: AppState.strategy,
                symbol: optimizeSymbol,
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

            // Store best params for "Apply" button
            AppState.bestSensitivityParams = bestResult.params;
        }

        // Show results block
        document.getElementById('optimize-results-placeholder').classList.add('hidden');
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
    document.getElementById('optimize-chart').innerHTML = ''; // Clear loading state

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
        paper_bgcolor: 'rgba(0,0,0,0)',
        plot_bgcolor: 'rgba(0,0,0,0)',
        font: { color: '#94a3b8' },
        margin: { t: 50, r: 80, b: 60, l: 60 },
        xaxis: { title: paramName, gridcolor: 'rgba(255, 255, 255, 0.05)' },
        yaxis: { title: 'Sharpe Ratio', gridcolor: 'rgba(255, 255, 255, 0.05)', side: 'left' },
        yaxis2: { title: 'Return (%)', overlaying: 'y', side: 'right', gridcolor: 'rgba(255, 255, 255, 0.05)' },
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

    document.getElementById('optimize-chart').innerHTML = ''; // Clear loading state

    Plotly.newPlot('optimize-chart', [{
        z: zData,
        x: xValues,
        y: yValues,
        type: 'heatmap',
        texttemplate: '%{z:.3f}',
        textfont: {
            family: 'Inter, sans-serif',
            size: 11,
            color: 'white',
            weight: 600 // Plotly uses integer weights or 'bold' string in newer versions, but font object usually takes styling
        },
        colorscale: [
            [0, '#ef4444'],
            [0.5, '#fbbf24'],
            [1, '#10b981']
        ],
        colorbar: { title: 'Sharpe' },
        hoverongaps: false
    }], {
        paper_bgcolor: 'rgba(0,0,0,0)',
        plot_bgcolor: 'rgba(0,0,0,0)',
        font: { color: '#94a3b8' },
        xaxis: { title: heatmapData.x_param, tickformat: '.1f', tickmode: 'array', tickvals: xValues },
        yaxis: { title: heatmapData.y_param, tickformat: '.1f', tickmode: 'array', tickvals: yValues },
        margin: { t: 50, r: 100, b: 60, l: 70 }
    }, { responsive: true });
}

// Old sensitivity applyBestParams removed - now using the one in Parameter Optimization section


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
    const badge = document.getElementById('detailed-sentiment-badge');
    badge.textContent = 'Loading...';
    badge.className = 'sentiment-badge';

    try {
        // Fetch 365 days history for chart + stats
        const res = await fetch(`${API_BASE}/sentiment/history?days=365`);
        const data = await res.json();

        if (data.success && data.data && data.data.length > 0) {
            const history = data.data;
            // Backend returns data.stats with pre-calculated metrics
            const stats = data.stats || {};

            // 1. Current Value (First item is usually latest from API, but let's verify sort)
            // Backend api.rs uses data.first() as current.
            const current = history[0];
            const value = current.value;

            // Update Badge
            badge.textContent = current.classification;
            badge.className = `sentiment-badge ${getSentimentClass(value)}`;

            // Update Large Gauge
            document.getElementById('gauge-large-value').textContent = value;
            document.getElementById('gauge-large-label').textContent = current.classification;
            document.getElementById('gauge-large-fill').style.width = `${value}%`;

            // 2. Key Stats
            // Daily Change (Current - Prev)
            let change = 0;
            if (history.length > 1) {
                change = value - history[1].value;
            }
            updateStatPill('sent-change', change, '', true);

            // 7-Day SMA
            const sma7 = stats.sma_7 !== undefined ? stats.sma_7 : calculateSMA(history, 7);
            updateStatPill('sent-sma7', sma7, '');

            // Momentum (Current - 7 Days Ago)
            const momentum = stats.momentum_7 !== undefined ? stats.momentum_7 : calculateMomentum(history, 7);
            updateStatPill('sent-momentum', momentum, '', true);

            // Dominance (Fear vs Greed days)
            const fearDays = stats.days_in_fear || 0;
            const greedDays = stats.days_in_greed || 0;
            const total = fearDays + greedDays + (stats.days_neutral || 0);
            let dominanceText = "Neutral";
            if (fearDays > greedDays) dominanceText = `${((fearDays / total) * 100).toFixed(0)}% Fear`;
            if (greedDays > fearDays) dominanceText = `${((greedDays / total) * 100).toFixed(0)}% Greed`;
            document.getElementById('sent-dominance').textContent = dominanceText;

            // 3. Render Chart
            renderSentimentChart(history);

        } else {
            console.warn('Sentiment data unavailable');
            badge.textContent = 'Unavailable';
        }
    } catch (e) {
        console.error('Failed to load sentiment:', e);
        badge.textContent = 'Error';
    }
}

function updateStatPill(id, value, suffix, colorize = false) {
    const el = document.getElementById(id);
    if (!el) return;

    if (value === undefined || value === null || isNaN(value)) {
        el.textContent = '--';
        return;
    }

    const formatted = Math.abs(value) < 10 ? value.toFixed(1) : value.toFixed(0);
    const sign = value > 0 ? '+' : '';
    el.textContent = `${sign}${formatted}${suffix}`;

    if (colorize) {
        el.style.color = value > 0 ? '#10b981' : (value < 0 ? '#ef4444' : 'var(--text-primary)');
    }
}

function calculateSMA(data, period) {
    if (data.length < period) return null;
    const slice = data.slice(0, period);
    const sum = slice.reduce((acc, curr) => acc + curr.value, 0);
    return sum / period;
}

function calculateMomentum(data, period) {
    if (data.length <= period) return null;
    return data[0].value - data[period].value;
}

function getSentimentClass(value) {
    if (value <= 25) return 'text-danger'; // Extreme Fear
    if (value <= 45) return 'text-warning'; // Fear
    if (value <= 55) return 'text-muted';  // Neutral
    if (value <= 75) return 'text-success'; // Greed
    return 'text-success'; // Extreme Greed
}

function renderSentimentChart(history) {
    // History needs to be reversed for chart (oldest to newest)
    const sorted = [...history].sort((a, b) => a.timestamp - b.timestamp);

    const x = sorted.map(d => new Date(d.timestamp * 1000));
    const y = sorted.map(d => d.value);
    const colors = y.map(v => {
        if (v <= 25) return '#ef4444';
        if (v <= 45) return '#f97316';
        if (v <= 55) return '#94a3b8';
        if (v <= 75) return '#84cc16';
        return '#10b981';
    });

    const trace = {
        x: x,
        y: y,
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: '#64748b', width: 2 },
        marker: {
            color: colors,
            size: 8,
            line: { color: 'white', width: 1 }
        },
        hovertemplate: '%{x|%b %d}<br>Score: %{y}<br>%{text}<extra></extra>',
        text: sorted.map(d => d.classification)
    };

    const layout = {
        paper_bgcolor: 'rgba(0,0,0,0)',
        plot_bgcolor: 'rgba(0,0,0,0)',
        height: 300,
        margin: { t: 20, r: 20, b: 40, l: 40 },
        font: { color: '#94a3b8' },
        xaxis: {
            gridcolor: 'rgba(255, 255, 255, 0.05)',
            showgrid: false
        },
        yaxis: {
            range: [0, 100],
            gridcolor: 'rgba(255, 255, 255, 0.05)',
            dtick: 25
        },
        shapes: [
            // Zones
            { type: 'rect', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: 0, y1: 25, fillcolor: 'rgba(239, 68, 68, 0.1)', line: { width: 0 } },
            { type: 'rect', xref: 'paper', yref: 'y', x0: 0, x1: 1, y0: 75, y1: 100, fillcolor: 'rgba(16, 185, 129, 0.1)', line: { width: 0 } }
        ]
    };

    Plotly.newPlot('sentiment-history-chart', [trace], layout, { responsive: true, displayModeBar: false });
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


// ===========================
// Comprehensive Optimization Workflow
// ===========================

let comprehensiveWorkflowResults = null;

async function runComprehensiveWorkflow() {
    const chartContainer = document.getElementById("optimize-chart");
    const resultsPlaceholder = document.getElementById("optimize-results-placeholder");

    // Show loading state
    chartContainer.innerHTML = `
        <div class="placeholder-content">
            <span class="placeholder-icon">⏳</span>
            <span>Running Comprehensive Optimization Workflow...</span>
            <span style="font-size: 12px; margin-top: 8px; opacity: 0.7;">
                Grid search + parameter dispersion + walk-forward + sensitivity analysis.
                This may take 30-60 seconds.
            </span>
        </div>`;

    resultsPlaceholder.innerHTML = `
        <span class="placeholder-icon">⏳</span>
        <span>Analyzing...</span>
    `;

    // Disable button
    const btn = document.querySelector("button[onclick=\"runComprehensiveWorkflow()\"]");
    if (btn) btn.disabled = true;

    try {
        const include3D = document.getElementById("include-3d-sensitivity").checked;
        
        // Get symbol from optimize tab
        const optimizeSymbol = document.getElementById("optimize-symbol").value;
        if (!optimizeSymbol) {
            throw new Error("Please select a trading symbol first");
        }
        
        // Update AppState with the selected symbol
        AppState.symbol = optimizeSymbol;
        
        const res = await fetch(`${API_BASE}/backtest/workflow`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                strategy: AppState.strategy,
                symbol: optimizeSymbol,
                interval: AppState.backtestInterval,
                days: 730, // Use 2 years for comprehensive analysis
                include_3d_sensitivity: include3D,
                train_window_days: 252,
                test_window_days: 63
            })
        });

        const data = await res.json();

        if (!data.success) {
            throw new Error(data.error || "Workflow failed");
        }

        // Store results
        comprehensiveWorkflowResults = data;

        // Update metrics
        const robustnessScore = data.robustness_score;
        const robustnessEl = document.getElementById("comp-robustness");
        robustnessEl.textContent = robustnessScore.toFixed(1);
        
        // Color code the robustness score
        if (robustnessScore >= 70) {
            robustnessEl.style.color = "#10b981"; // Green
        } else if (robustnessScore >= 50) {
            robustnessEl.style.color = "#f59e0b"; // Orange
        } else {
            robustnessEl.style.color = "#ef4444"; // Red
        }

        document.getElementById("comp-sharpe").textContent = data.best_sharpe.toFixed(2);
        document.getElementById("comp-wf-stability").textContent = data.walk_forward_stability_score.toFixed(2);
        document.getElementById("comp-param-cv").textContent = data.parameter_dispersion.sharpe_cv.toFixed(2);
        document.getElementById("comp-positive-pct").textContent = data.parameter_dispersion.positive_sharpe_pct.toFixed(0) + "%";
        document.getElementById("comp-iterations").textContent = data.sweep_results.length;

        // Show optimized parameters
        const paramSummary = document.getElementById("comp-param-summary");
        let paramsHtml = "<div style=\"margin-top: 12px; padding: 12px; background: rgba(139, 92, 246, 0.1); border-radius: 8px;\">";
        paramsHtml += "<strong>Optimized Parameters:</strong><br>";
        for (const [key, value] of Object.entries(data.optimized_params)) {
            paramsHtml += `<span style=\"display: inline-block; margin: 4px 8px 4px 0;\">${key}: <strong>${value.toFixed(2)}</strong></span>`;
        }
        paramsHtml += "</div>";
        paramSummary.innerHTML = paramsHtml;

        // Show results block
        resultsPlaceholder.classList.add("hidden");
        document.getElementById("comprehensive-results").classList.remove("hidden");
        document.getElementById("wfa-results").classList.add("hidden");
        document.getElementById("sensitivity-results").classList.add("hidden");

        // Render visualization
        if (data.sensitivity_heatmap && include3D) {
            renderSensitivityHeatmap(data.sensitivity_heatmap);
            document.getElementById("optimize-chart-title").textContent = "3D Parameter Sensitivity Heatmap";
        } else {
            renderParameterSweep(data.sweep_results);
            document.getElementById("optimize-chart-title").textContent = "Parameter Sweep Results";
        }

    } catch (e) {
        console.error("Comprehensive Workflow Error:", e);
        chartContainer.innerHTML = `
            <div class="placeholder-content" style="color: #ef4444;">
                <span class="placeholder-icon">⚠️</span>
                <span>Workflow Failed</span>
                <span style="font-size: 14px; margin-top: 8px; max-width: 80%; text-align: center;">${e.message}</span>
                <button class="btn-primary" style="margin-top: 16px;" onclick="runComprehensiveWorkflow()">Try Again</button>
            </div>`;

        resultsPlaceholder.innerHTML = `
            <span class="placeholder-icon" style="color: #ef4444;">⚠️</span>
            <span style="color: #ef4444;">Workflow Failed</span>
        `;
    } finally {
        if (btn) btn.disabled = false;
    }
}

function renderSensitivityHeatmap(heatmap) {
    const data = [{
        z: heatmap.sharpe_matrix,
        x: heatmap.x_values,
        y: heatmap.y_values,
        type: "heatmap",
        colorscale: "RdYlGn",
        colorbar: {
            title: "Sharpe Ratio"
        }
    }];

    const layout = {
        title: `${heatmap.x_param} vs ${heatmap.y_param}`,
        xaxis: { title: heatmap.x_param },
        yaxis: { title: heatmap.y_param },
        paper_bgcolor: "rgba(0,0,0,0)",
        plot_bgcolor: "rgba(0,0,0,0)",
        font: { color: "#e5e7eb" }
    };

    Plotly.newPlot("optimize-chart", data, layout, { responsive: true });
}

function renderParameterSweep(sweepResults) {
    // Show parameter sweep as scatter plot
    const data = [{
        x: sweepResults.map(r => r.sharpe),
        y: sweepResults.map(r => r.total_return * 100),
        mode: "markers",
        type: "scatter",
        marker: {
            size: 8,
            color: sweepResults.map(r => r.score),
            colorscale: "Viridis",
            colorbar: {
                title: "Composite Score"
            }
        },
        text: sweepResults.map(r => `Score: ${r.score.toFixed(2)}<br>Trades: ${r.total_trades}`),
        hovertemplate: "%{text}<extra></extra>"
    }];

    const layout = {
        title: "Parameter Combinations (Sharpe vs Return)",
        xaxis: { title: "Sharpe Ratio" },
        yaxis: { title: "Total Return (%)" },
        paper_bgcolor: "rgba(0,0,0,0)",
        plot_bgcolor: "rgba(0,0,0,0)",
        font: { color: "#e5e7eb" }
    };

    Plotly.newPlot("optimize-chart", data, layout, { responsive: true });
}

function applyOptimizedParams() {
    if (!comprehensiveWorkflowResults) {
        alert("No optimization results available");
        return;
    }

    // Apply params to AppState
    AppState.params = comprehensiveWorkflowResults.optimized_params;

    // Update UI inputs
    const strategyParams = STRATEGY_PARAMS[AppState.strategy] || [];
    strategyParams.forEach(param => {
        const value = comprehensiveWorkflowResults.optimized_params[param.name];
        if (value !== undefined) {
            const input = document.getElementById(`param-${param.name}`);
            if (input) {
                input.value = value;
            }
        }
    });

    // Update backtest summary
    updateBacktestSummary();

    // Navigate to backtest tab
    switchTab("backtest");
    
    alert(`✓ Optimized parameters applied!\n\nRobustness Score: ${comprehensiveWorkflowResults.robustness_score.toFixed(1)}/100\n\nClick "Run Backtest" to see full results with these parameters.`);
}


function goToBacktestWithOptimizedParams() {
    if (!comprehensiveWorkflowResults) {
        alert("No optimization results available");
        return;
    }

    // Apply params to AppState
    AppState.params = comprehensiveWorkflowResults.optimized_params;
    
    // Ensure symbol is set from optimize tab
    const optimizeSymbol = document.getElementById("optimize-symbol").value;
    if (optimizeSymbol) {
        AppState.symbol = optimizeSymbol;
    }

    // Update context bar and backtest summary
    updateContextBar();
    updateBacktestSummary();

    // Navigate to backtest tab
    switchTab("backtest");
}

// Initialize symbol selector for optimize tab
function initOptimizeSymbolSelect() {
    const wrapper = document.getElementById('optimize-symbol-select-wrapper');
    const trigger = document.getElementById('optimize-symbol-trigger');
    const optionsList = document.getElementById('optimize-symbol-options-list');
    const searchInput = document.getElementById('optimize-symbol-search-input');
    const hiddenInput = document.getElementById('optimize-symbol');
    const selectedText = document.getElementById('optimize-selected-symbol-text');

    if (!wrapper || !trigger) return; // Exit if elements don't exist yet

    // Toggle dropdown
    trigger.addEventListener('click', () => {
        wrapper.classList.toggle('open');
        if (wrapper.classList.contains('open')) {
            searchInput.focus();
        }
    });

    // Close when clicking outside
    document.addEventListener('click', (e) => {
        if (!wrapper.contains(e.target)) {
            wrapper.classList.remove('open');
        }
    });

    // Filter function
    function filterSymbols(query) {
        const q = query.toLowerCase();
        const POPULAR = ['BTC', 'ETH', 'SOL', 'XRP', 'ADA', 'DOGE', 'AVAX', 'DOT', 'LINK', 'MATIC'];

        let filtered = allSymbols.filter(s => s.toLowerCase().includes(q));

        // Sort: Popular first, then alphabetical
        filtered.sort((a, b) => {
            const aPop = POPULAR.indexOf(a);
            const bPop = POPULAR.indexOf(b);

            if (aPop !== -1 && bPop !== -1) return aPop - bPop;
            if (aPop !== -1) return -1;
            if (bPop !== -1) return 1;
            return a.localeCompare(b);
        });

        renderOptions(filtered);
    }

    // Render options to the list
    function renderOptions(symbols) {
        optionsList.innerHTML = '';

        if (symbols.length === 0) {
            optionsList.innerHTML = '<div class="option" style="cursor: default; opacity: 0.5;">No matches found</div>';
            return;
        }

        // Limit rendering for performance if list is huge
        const displaySymbols = symbols.slice(0, 100);

        displaySymbols.forEach(symbol => {
            const div = document.createElement('div');
            div.className = 'option';
            if (symbol === AppState.symbol) div.classList.add('selected');

            div.innerHTML = `
                <span class="option-ticker">${symbol}</span>
                <span class="option-name">USDT</span>
            `;

            div.addEventListener('click', () => {
                AppState.symbol = symbol;
                hiddenInput.value = symbol;
                selectedText.textContent = symbol;
                wrapper.classList.remove('open');
                updateContextBar();
            });

            optionsList.appendChild(div);
        });
    }

    // Search as you type
    searchInput.addEventListener('input', (e) => {
        filterSymbols(e.target.value);
    });

    // Initial render
    filterSymbols('');
}
