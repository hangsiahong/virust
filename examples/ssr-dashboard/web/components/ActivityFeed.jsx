// ActivityFeed.jsx - Server-side activity feed component
export default function ActivityFeed({ activities }) {
  if (!activities || activities.length === 0) {
    return (
      <div style={{
        padding: '40px',
        background: 'white',
        borderRadius: '12px',
        textAlign: 'center',
        color: '#666',
        border: '1px solid #e5e7eb'
      }}>
        No recent activity
      </div>
    );
  }

  return (
    <div style={{
      background: 'white',
      borderRadius: '12px',
      border: '1px solid #e5e7eb',
      boxShadow: '0 2px 8px rgba(0,0,0,0.06)',
      overflow: 'hidden'
    }}>
      {activities.map((activity, index) => (
        <div
          key={activity.id}
          style={{
            padding: '20px',
            borderBottom: index < activities.length - 1 ? '1px solid #f0f0f0' : 'none',
            display: 'flex',
            alignItems: 'flex-start',
            gap: '15px',
            transition: 'background 0.2s',
            cursor: 'default'
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = '#f9fafb';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = 'white';
          }}
        >
          <div style={{
            fontSize: '1.5rem',
            flexShrink: 0,
            width: '40px',
            height: '40px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: '#f3f4f6',
            borderRadius: '8px'
          }}>
            {activity.icon}
          </div>

          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{
              fontSize: '0.95rem',
              color: '#1a202c',
              marginBottom: '6px',
              lineHeight: '1.5'
            }}>
              {activity.message}
            </div>
            <div style={{
              fontSize: '0.8rem',
              color: '#888',
              fontWeight: '500'
            }}>
              {activity.time}
            </div>
          </div>

          {/* Activity type badge */}
          <div style={{
            flexShrink: 0,
            padding: '4px 10px',
            borderRadius: '12px',
            fontSize: '0.7rem',
            fontWeight: '600',
            textTransform: 'uppercase',
            letterSpacing: '0.3px',
            background: getActivityTypeBg(activity.type),
            color: getActivityTypeColor(activity.type)
          }}>
            {activity.type}
          </div>
        </div>
      ))}
    </div>
  );
}

function getActivityTypeBg(type) {
  const colors = {
    user: '#dbeafe',
    purchase: '#d1fae5',
    alert: '#fef3c7',
    success: '#d1fae5'
  };
  return colors[type] || '#f3f4f6';
}

function getActivityTypeColor(type) {
  const colors = {
    user: '#1e40af',
    purchase: '#065f46',
    alert: '#92400e',
    success: '#065f46'
  };
  return colors[type] || '#374151';
}
