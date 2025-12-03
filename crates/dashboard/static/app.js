const API_BASE = 'http://localhost:8080/api';

// Format currency
const formatMoney = (value) => {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD'
    }).format(value);
};

// Format percentage
const formatPercent = (value) => {
    return `${value >= 0 ? '+' : ''}${value.toFixed(2)}%`;
};

// Apply color class based on value
const colorClass = (value) => value >= 0 ? 'positive' : 'negative';

// Initialize Chart
let equityChart;

function initChart() {
    const ctx = document.getElementById('equityChart').getContext('2d');
    equityChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: ['Day 1', 'Day 2', 'Day 3', 'Day 4', 'Day 5', 'Day 6', 'Day 7'],
            datasets: [{
                label: 'Equity',
                data: [100000, 101200, 100800, 102500, 103000, 104200, 105432],
                borderColor: '#3b82f6',
                backgroundColor: 'rgba(59, 130, 246, 0.1)',
                tension: 0.4,
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: { display: false }
            },
            scales: {
                y: {
                    grid: { color: '#334155' },
                    ticks: { color: '#94a3b8' }
                },
                x: {
                    grid: { display: false },
                    ticks: { color: '#94a3b8' }
                }
            }
        }
    });
}

// Fetch and update data
async function updateDashboard() {
    try {
        // 1. Portfolio Summary
        const portfolioRes = await fetch(`${API_BASE}/portfolio`);
        const portfolio = await portfolioRes.json();

        document.getElementById('total-value').textContent = formatMoney(portfolio.total_value);
        document.getElementById('cash-balance').textContent = formatMoney(portfolio.cash);

        const pnlEl = document.getElementById('total-pnl');
        pnlEl.textContent = formatMoney(portfolio.pnl);
        pnlEl.className = `value ${colorClass(portfolio.pnl)}`;

        const pnlPctEl = document.getElementById('pnl-percent');
        pnlPctEl.textContent = formatPercent(portfolio.pnl_percent);
        pnlPctEl.className = `sub-value ${colorClass(portfolio.pnl_percent)}`;

        // 2. Performance Metrics
        const perfRes = await fetch(`${API_BASE}/performance`);
        const perf = await perfRes.json();

        document.getElementById('sharpe-ratio').textContent = perf.sharpe_ratio.toFixed(2);
        document.getElementById('win-rate').textContent = `${perf.win_rate.toFixed(1)}%`;
        document.getElementById('max-drawdown').textContent = `${perf.max_drawdown.toFixed(1)}%`;
        document.getElementById('total-trades').textContent = perf.total_trades;

        // 3. Active Positions
        const posRes = await fetch(`${API_BASE}/positions`);
        const positions = await posRes.json();

        const posTable = document.getElementById('positions-table').querySelector('tbody');
        posTable.innerHTML = positions.map(p => `
            <tr>
                <td>${p.symbol}</td>
                <td>${p.quantity}</td>
                <td>${formatMoney(p.entry_price)}</td>
                <td>${formatMoney(p.current_price)}</td>
                <td class="${colorClass(p.pnl)}">${formatMoney(p.pnl)}</td>
                <td class="${colorClass(p.pnl_percent)}">${formatPercent(p.pnl_percent)}</td>
            </tr>
        `).join('');

        // 4. Recent Orders
        const ordRes = await fetch(`${API_BASE}/orders`);
        const orders = await ordRes.json();

        const ordTable = document.getElementById('orders-table').querySelector('tbody');
        ordTable.innerHTML = orders.map(o => `
            <tr>
                <td>${new Date(o.timestamp).toLocaleTimeString()}</td>
                <td>${o.id.substring(0, 8)}...</td>
                <td>${o.symbol}</td>
                <td class="${o.side === 'Buy' ? 'buy-side' : 'sell-side'}">${o.side}</td>
                <td>${o.order_type}</td>
                <td>${o.quantity}</td>
                <td>${o.price ? formatMoney(o.price) : 'Market'}</td>
                <td>${o.status}</td>
            </tr>
        `).join('');

    } catch (error) {
        console.error('Error fetching data:', error);
    }
}

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    initChart();
    updateDashboard();

    // Auto-refresh every 5 seconds
    setInterval(updateDashboard, 5000);
});
