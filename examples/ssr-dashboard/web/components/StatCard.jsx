// StatCard.jsx - Server-side metric card component
export default function StatCard({ label, value, change, trend }) {
  // Format value based on label
  const formatValue = (val, lbl) => {
    if (lbl.toLowerCase().includes('revenue')) {
      return '$' + (val / 1000).toFixed(1) + 'k';
    }
    if (lbl.toLowerCase().includes('rate')) {
      return val + '%';
    }
    if (lbl.toLowerCase().includes('duration')) {
      return val + ' min';
    }
    if (val >= 1000) {
      return (val / 1000).toFixed(1) + 'k';
    }
    return val.toLocaleString();
  };

  const trendColor = trend === 'up' ? '#10b981' : '#ef4444';
  const trendIcon = trend === 'up' ? '↑' : '↓';
  const changeColor = change >= 0 ? '#10b981' : '#ef4444';
  const changeIcon = change >= 0 ? '↑' : '↓';

  return (
    <div style={{
      background: 'white',
      padding: '25px',
      borderRadius: '12px',
      boxShadow: '0 2px 8px rgba(0,0,0,0.06)',
      border: '1px solid #e5e7eb',
      transition: 'all 0.2s',
      cursor: 'default'
    }}
    onMouseEnter={(e) => {
      e.currentTarget.style.transform = 'translateY(-2px)';
      e.currentTarget.style.boxShadow = '0 4px 16px rgba(0,0,0,0.1)';
    }}
    onMouseLeave={(e) => {
      e.currentTarget.style.transform = 'translateY(0)';
      e.currentTarget.style.boxShadow = '0 2px 8px rgba(0,0,0,0.06)';
    }}
    >
      <div style={{
        fontSize: '0.85rem',
        color: '#666',
        marginBottom: '12px',
        textTransform: 'uppercase',
        letterSpacing: '0.5px',
        fontWeight: '600'
      }}>
        {label}
      </div>

      <div style={{
        fontSize: '2.2rem',
        fontWeight: 'bold',
        color: '#1a202c',
        marginBottom: '10px',
        letterSpacing: '-0.5px'
      }}>
        {formatValue(value, label)}
      </div>

      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        fontSize: '0.9rem',
        fontWeight: '500',
        color: changeColor
      }}>
        <span>{changeIcon}</span>
        <span>{Math.abs(change)}%</span>
        <span style={{ color: '#888', fontWeight: '400' }}>from last month</span>
      </div>

      {/* Mini trend indicator */}
      <div style={{
        marginTop: '15px',
        height: '4px',
        background: '#f0f0f0',
        borderRadius: '2px',
        overflow: 'hidden'
      }}>
        <div style={{
          width: '75%',
          height: '100%',
          background: trend === 'up'
            ? 'linear-gradient(90deg, #10b981 0%, #34d399 100%)'
            : 'linear-gradient(90deg, #ef4444 0%, #f87171 100%)',
          borderRadius: '2px',
          transition: 'width 0.3s'
        }}></div>
      </div>
    </div>
  );
}
