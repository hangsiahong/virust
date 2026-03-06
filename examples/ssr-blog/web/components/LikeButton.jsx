// LikeButton.jsx - Client-side interactive like button
'use client';

import { useState } from 'react';

export default function LikeButton({ initialLikes = 0 }) {
  const [likes, setLikes] = useState(initialLikes);
  const [isLiked, setIsLiked] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);

  const handleLike = () => {
    if (isAnimating) return;

    setIsAnimating(true);
    setIsLiked(!isLiked);
    setLikes(prev => isLiked ? prev - 1 : prev + 1);

    setTimeout(() => {
      setIsAnimating(false);
    }, 300);
  };

  return (
    <div style={{ display: 'inline-block' }}>
      <button
        onClick={handleLike}
        disabled={isAnimating}
        style={{
          padding: '12px 24px',
          fontSize: '1rem',
          fontWeight: '600',
          background: isLiked
            ? 'linear-gradient(135deg, #ef4444 0%, #dc2626 100%)'
            : 'white',
          color: isLiked ? 'white' : '#ef4444',
          border: isLiked ? 'none' : '2px solid #ef4444',
          borderRadius: '50px',
          cursor: isAnimating ? 'not-allowed' : 'pointer',
          transition: 'all 0.3s',
          opacity: isAnimating ? 0.8 : 1,
          transform: isAnimating ? 'scale(1.1)' : 'scale(1)',
          boxShadow: isLiked
            ? '0 4px 12px rgba(239, 68, 68, 0.4)'
            : '0 2px 8px rgba(0,0,0,0.1)',
          display: 'flex',
          alignItems: 'center',
          gap: '10px'
        }}
      >
        <span style={{
          fontSize: '1.3rem',
          transform: isAnimating ? 'scale(1.2)' : 'scale(1)',
          transition: 'transform 0.3s'
        }}>
          {isLiked ? '❤️' : '🤍'}
        </span>
        <span>{likes} {likes === 1 ? 'Like' : 'Likes'}</span>
      </button>
      {isAnimating && (
        <div style={{
          marginTop: '10px',
          fontSize: '0.85rem',
          color: isLiked ? '#ef4444' : '#666',
          fontWeight: '500'
        }}>
          {isLiked ? '❤️ You liked this!' : '💔 Like removed'}
        </div>
      )}
    </div>
  );
}
