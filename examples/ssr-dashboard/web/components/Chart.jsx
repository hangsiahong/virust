// Chart.jsx - Server-side chart component with data visualization
export default function Chart({ data }) {
  if (!data || data.length === 0) {
    return (
      <div style={{
        padding: '40px',
        background: 'white',
        borderRadius: '12px',
        textAlign: 'center',
        color: '#666',
        border: '1px solid #e5e7eb'
      }}>
        No chart data available
      </div>
    );
  }

  const maxValue = Math.max(...data.map(d => d.value));
  const minValue = Math.min(...data.map(d => d.value));

  return (
    <div style={{
      background: 'white',
      padding: '30px',
      borderRadius: '12px',
      border: '1px solid #e5e7eb',
      boxShadow: '0 2px 8px rgba(0,0,0,0.06)'
    }}>
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '30px'
      }}>
        <h3 style={{
          fontSize: '1.1rem',
          color: '#333',
          fontWeight: '700'
        }}>
          Weekly Traffic
        </h3>
        <div style={{
          display: 'flex',
          gap: '20px',
          fontSize: '0.9rem'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div style={{
              width: '12px',
              height: '12px',
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              borderRadius: '3px'
            }}></div>
            <span style={{ color: '#666' }}>Page Views</span>
          </div>
        </div>
      </div>

      {/* Bar Chart */}
      <div style={{
        display: 'flex',
        alignItems: 'flex-end',
        justifyContent: 'space-between',
        height: '200px',
        gap: '15px',
        padding: '20px 0',
        borderBottom: '1px solid #e5e7eb',
        borderLeft: '1px solid #e5e7eb',
        position: 'relative'
      }}>
        {/* Y-axis labels */}
        <div style={{
          position: 'absolute',
          left: '-35px',
          top: 0,
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          fontSize: '0.75rem',
          color: '#888',
          fontWeight: '500'
        }}>
          <span>{(maxValue / 1000).toFixed(1)}k</span>
          <span>{((maxValue + minValue) / 2000).toFixed(1)}k</span>
          <span>0</span>
        </div>

        {data.map((point, index) => {
          const height = (point.value / maxValue) * 100;
          const isHighest = point.value === maxValue;

          return (
            <div
              key={index}
              style={{
                flex: 1,
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                gap: '10px'
              }}
            >
              {/* Value label */}
              <div style={{
                fontSize: '0.8rem',
                fontWeight: '600',
                color: isHighest ? '#667eea' : '#666',
                opacity: 0,
                transition: 'opacity 0.2s'
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.opacity = '1';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.opacity = '0';
              }}
              >
                {(point.value / 1000).toFixed(1)}k
              </div>

              {/* Bar */}
              <div style={{
                width: '100%',
                height: `${height}%`,
                background: isHighest
                  ? 'linear-gradient(180deg, #667eea 0%, #764ba2 100%)'
                  : 'linear-gradient(180deg, #667eea80 0%, #764ba280 100%)',
                borderRadius: '6px 6px 0 0',
                transition: 'all 0.3s',
                position: 'relative',
                cursor: 'pointer',
                boxShadow: isHighest ? '0 4px 12px rgba(102, 126, 234, 0.3)' : 'none'
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.transform = 'translateY(-5px)';
                e.currentTarget.style.boxShadow = '0 6px 16px rgba(102, 126, 234, 0.4)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = 'translateY(0)';
                e.currentTarget.style.boxShadow = isHighest ? '0 4px 12px rgba(102, 126, 234, 0.3)' : 'none';
              }}
              title={`${point.label}: ${point.value.toLocaleString()} views`}
              >
                {/* Shine effect */}
                <div style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  right: 0,
                  height: '40%',
                  background: 'linear-gradient(180deg, rgba(255,255,255,0.2) 0%, transparent 100%)',
                  borderRadius: '6px 6px 0 0'
                }}></div>
              </div>

              {/* X-axis label */}
              <div style={{
                fontSize: '0.85rem',
                color: '#666',
                fontWeight: '500',
                marginTop: '5px'
              }}>
                {point.label}
              </div>
            </div>
          );
        })}
      </div>

      {/* Summary stats */}
      <div style={{
        marginTop: '25px',
        display: 'flex',
        justifyContent: 'space-around',
        paddingTop: '20px',
        borderTop: '1px solid #f0f0f0'
      }}>
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '0.8rem', color: '#888', marginBottom: '5px' }}>
            Total Views
          </div>
          <div style={{ fontSize: '1.2rem', fontWeight: 'bold', color: '#1a202c' }}>
            {data.reduce((sum, d) => sum + d.value, 0).toLocaleString()}
          </div>
        </div>
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '0.8rem', color: '#888', marginBottom: '5px' }}>
            Average
          </div>
          <div style={{ fontSize: '1.2rem', fontWeight: 'bold', color: '#1a202c' }}>
            {(data.reduce((sum, d) => sum + d.value, 0) / data.length / 1000).toFixed(1)}k
          </div>
        </div>
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '0.8rem', color: '#888', marginBottom: '5px' }}>
            Peak Day
          </div>
          <div style={{ fontSize: '1.2rem', fontWeight: 'bold', color: '#667eea' }}>
            {data.find(d => d.value === maxValue)?.label || 'N/A'}
          </div>
        </div>
      </div>
    </div>
  );
}
