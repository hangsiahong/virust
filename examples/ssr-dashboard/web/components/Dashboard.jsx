// Dashboard.jsx - Server-side rendered dashboard with data visualization
import StatCard from './StatCard';
import ActivityFeed from './ActivityFeed';
import Chart from './Chart';
import RefreshButton from './RefreshButton';

export default function Dashboard({ title, metrics, chartData, activities, lastUpdated }) {
  return (
    <div style={{
      maxWidth: '1400px',
      margin: '0 auto',
      padding: '30px',
      fontFamily: 'system-ui, -apple-system, sans-serif',
      background: '#f5f7fa',
      minHeight: '100vh'
    }}>
      {/* Header */}
      <header style={{
        marginBottom: '40px',
        paddingBottom: '30px',
        borderBottom: '2px solid #e1e8ed',
        background: 'white',
        padding: '30px',
        borderRadius: '12px',
        boxShadow: '0 2px 8px rgba(0,0,0,0.06)'
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <h1 style={{
              fontSize: '2.5rem',
              marginBottom: '10px',
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text',
              fontWeight: '800'
            }}>
              {title || 'Analytics Dashboard'}
            </h1>
            <p style={{ color: '#666', fontSize: '1.1rem', marginTop: '10px' }}>
              Real-time metrics and insights
            </p>
            <div style={{
              marginTop: '15px',
              fontSize: '0.85rem',
              color: '#888',
              display: 'flex',
              alignItems: 'center',
              gap: '8px'
            }}>
              <span style={{
                display: 'inline-block',
                width: '8px',
                height: '8px',
                background: '#10b981',
                borderRadius: '50%',
                animation: 'pulse 2s infinite'
              }}></span>
              Last updated: {lastUpdated}
            </div>
          </div>
          <div style={{ display: 'flex', gap: '15px', alignItems: 'center' }}>
            <RefreshButton />
          </div>
        </div>
      </header>

      {/* Metrics Grid */}
      <main>
        <section style={{ marginBottom: '40px' }}>
          <h2 style={{
            fontSize: '1.5rem',
            marginBottom: '25px',
            color: '#1a202c',
            fontWeight: '700'
          }}>
            Key Metrics
          </h2>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
            gap: '20px'
          }}>
            {metrics && metrics.length > 0 ? (
              metrics.map((metric, index) => (
                <StatCard
                  key={index}
                  label={metric.label}
                  value={metric.value}
                  change={metric.change}
                  trend={metric.trend}
                />
              ))
            ) : (
              <div style={{
                gridColumn: '1 / -1',
                padding: '40px',
                background: 'white',
                borderRadius: '8px',
                textAlign: 'center',
                color: '#666'
              }}>
                No metrics available
              </div>
            )}
          </div>
        </section>

        {/* Chart Section */}
        <section style={{ marginBottom: '40px' }}>
          <h2 style={{
            fontSize: '1.5rem',
            marginBottom: '25px',
            color: '#1a202c',
            fontWeight: '700'
          }}>
            Weekly Overview
          </h2>
          <Chart data={chartData || []} />
        </section>

        {/* Two Column Layout */}
        <div style={{
          display: 'grid',
          gridTemplateColumns: '2fr 1fr',
          gap: '20px',
          marginBottom: '40px'
        }}>
          {/* Activity Feed */}
          <section>
            <h2 style={{
              fontSize: '1.5rem',
              marginBottom: '25px',
              color: '#1a202c',
              fontWeight: '700'
            }}>
              Recent Activity
            </h2>
            <ActivityFeed activities={activities || []} />
          </section>

          {/* Quick Start Guide */}
          <section style={{
            background: 'linear-gradient(135deg, #667eea10 0%, #764ba210 100%)',
            padding: '30px',
            borderRadius: '12px',
            border: '2px solid #667eea30',
            height: 'fit-content'
          }}>
            <h3 style={{
              fontSize: '1.3rem',
              marginBottom: '20px',
              color: '#333',
              fontWeight: '700'
            }}>
              SSR Architecture
            </h3>
            <ul style={{
              lineHeight: '1.8',
              color: '#555',
              paddingLeft: '20px',
              fontSize: '0.95rem'
            }}>
              <li style={{ marginBottom: '12px' }}>
                <strong style={{ color: '#667eea' }}>Server Rendering:</strong> This HTML is generated on the server
              </li>
              <li style={{ marginBottom: '12px' }}>
                <strong style={{ color: '#667eea' }}>Data Fetching:</strong> Backend fetches data and passes it as props
              </li>
              <li style={{ marginBottom: '12px' }}>
                <strong style={{ color: '#667eea' }}>Client Components:</strong> Interactive elements use 'use client'
              </li>
              <li style={{ marginBottom: '12px' }}>
                <strong style={{ color: '#667eea' }}>Hybrid Approach:</strong> Best of both worlds
              </li>
              <li style={{ marginBottom: '12px' }}>
                <strong style={{ color: '#667eea' }}>Performance:</strong> Fast initial load, excellent SEO
              </li>
            </ul>
            <div style={{
              marginTop: '20px',
              padding: '15px',
              background: 'white',
              borderRadius: '8px',
              fontSize: '0.85rem',
              color: '#666'
            }}>
              <strong>💡 Pro Tip:</strong> View the page source to see the server-rendered HTML!
            </div>
          </section>
        </div>
      </main>

      {/* Footer */}
      <footer style={{
        marginTop: '60px',
        padding: '30px',
        background: 'white',
        borderRadius: '12px',
        textAlign: 'center',
        color: '#666',
        border: '1px solid #e5e7eb',
        boxShadow: '0 2px 8px rgba(0,0,0,0.06)'
      }}>
        <p style={{ marginBottom: '10px', fontSize: '0.95rem' }}>
          Built with <strong>Virust</strong> - Server-Side Rendering for Rust
        </p>
        <p style={{ fontSize: '0.85rem', color: '#888' }}>
          Data fetched from backend • Components rendered on server • Interactive features on client
        </p>
      </footer>

      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }
      `}</style>
    </div>
  );
}
