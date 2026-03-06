// RefreshButton.jsx - Client-side interactive refresh button
'use client';

import { useState } from 'react';

export default function RefreshButton() {
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [lastRefresh, setLastRefresh] = useState(null);
  const [rotation, setRotation] = useState(0);

  const handleRefresh = () => {
    if (isRefreshing) return;

    setIsRefreshing(true);

    // Simulate a refresh action
    setTimeout(() => {
      setIsRefreshing(false);
      setLastRefresh(new Date().toLocaleTimeString());
      setRotation(rotation + 360);
    }, 1500);
  };

  return (
    <div>
      <button
        onClick={handleRefresh}
        disabled={isRefreshing}
        style={{
          padding: '12px 24px',
          fontSize: '1rem',
          fontWeight: '600',
          background: isRefreshing
            ? 'linear-gradient(135deg, #9ca3af 0%, #6b7280 100%)'
            : 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
          color: 'white',
          border: 'none',
          borderRadius: '8px',
          cursor: isRefreshing ? 'not-allowed' : 'pointer',
          transition: 'all 0.3s',
          opacity: isRefreshing ? 0.8 : 1,
          boxShadow: isRefreshing
            ? 'none'
            : '0 4px 12px rgba(102, 126, 234, 0.3)',
          display: 'flex',
          alignItems: 'center',
          gap: '10px'
        }}
        onMouseEnter={(e) => {
          if (!isRefreshing) {
            e.currentTarget.style.transform = 'translateY(-2px)';
            e.currentTarget.style.boxShadow = '0 6px 16px rgba(102, 126, 234, 0.4)';
          }
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.transform = 'translateY(0)';
          if (!isRefreshing) {
            e.currentTarget.style.boxShadow = '0 4px 12px rgba(102, 126, 234, 0.3)';
          }
        }}
      >
        <span style={{
          display: 'inline-block',
          fontSize: '1.2rem',
          transform: `rotate(${rotation}deg)`,
          transition: 'transform 0.5s ease-in-out'
        }}>
          🔄
        </span>
        <span>{isRefreshing ? 'Refreshing...' : 'Refresh Data'}</span>
      </button>

      {lastRefresh && (
        <div style={{
          marginTop: '12px',
          padding: '8px 12px',
          background: '#d1fae5',
          color: '#065f46',
          borderRadius: '6px',
          fontSize: '0.85rem',
          fontWeight: '500',
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
          animation: 'fadeIn 0.3s ease-in'
        }}>
          <span>✅</span>
          <span>Last refreshed: {lastRefresh}</span>
        </div>
      )}

      <style>{`
        @keyframes fadeIn {
          from {
            opacity: 0;
            transform: translateY(-5px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
      `}</style>
    </div>
  );
}
