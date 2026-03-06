// BlogPost.jsx - Server-side rendered blog post
import LikeButton from './LikeButton';

export default function BlogPost({ id, title, content, author, date, likes }) {
  return (
    <div style={{
      maxWidth: '800px',
      margin: '0 auto',
      padding: '40px 20px',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      <header style={{
        marginBottom: '40px',
        paddingBottom: '30px',
        borderBottom: '2px solid #eaeaea'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '15px', marginBottom: '20px' }}>
          <div style={{
            width: '50px',
            height: '50px',
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            borderRadius: '10px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '24px',
            color: 'white',
            fontWeight: 'bold'
          }}>
            V
          </div>
          <a
            href="/"
            style={{
              fontSize: '1.5rem',
              fontWeight: '700',
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text',
              textDecoration: 'none'
            }}
          >
            Virust Blog
          </a>
        </div>

        <div style={{
          padding: '8px 16px',
          background: 'linear-gradient(135deg, #667eea20 0%, #764ba220 100%)',
          color: '#667eea',
          borderRadius: '20px',
          fontSize: '0.85rem',
          fontWeight: '600',
          textTransform: 'uppercase',
          letterSpacing: '0.5px',
          display: 'inline-block',
          marginBottom: '20px'
        }}>
          Blog Post
        </div>

        <h1 style={{
          fontSize: '2.5rem',
          lineHeight: '1.2',
          marginBottom: '20px',
          color: '#1a202c',
          fontWeight: '800'
        }}>
          {title}
        </h1>

        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: '15px',
          color: '#666',
          fontSize: '0.95rem'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div style={{
              width: '28px',
              height: '28px',
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              borderRadius: '50%',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'white',
              fontSize: '0.75rem',
              fontWeight: 'bold'
            }}>
              {author.charAt(0)}
            </div>
            <span style={{ fontWeight: '600' }}>{author}</span>
          </div>
          <span>•</span>
          <span>{date}</span>
        </div>
      </header>

      <main>
        <article
          style={{
            fontSize: '1.1rem',
            lineHeight: '1.8',
            color: '#2d3748'
          }}
          dangerouslySetInnerHTML={{ __html: content }}
        />

        <section style={{
          marginTop: '60px',
          padding: '30px',
          background: 'linear-gradient(135deg, #667eea10 0%, #764ba210 100%)',
          borderRadius: '12px',
          border: '2px solid #667eea30'
        }}>
          <h3 style={{
            fontSize: '1.4rem',
            marginBottom: '15px',
            color: '#333'
          }}>
            Did you enjoy this post?
          </h3>
          <p style={{ color: '#666', marginBottom: '20px', lineHeight: '1.6' }}>
            Show your support by liking it! This button is a client component with React hooks.
          </p>
          <LikeButton initialLikes={likes || 0} />
        </section>

        <section style={{
          marginTop: '50px',
          padding: '30px',
          background: '#f9f9f9',
          borderRadius: '12px',
          border: '1px solid #e5e7eb'
        }}>
          <h3 style={{
            fontSize: '1.4rem',
            marginBottom: '20px',
            color: '#333'
          }}>
            About Virust SSR
          </h3>
          <ul style={{
            lineHeight: '1.8',
            color: '#555',
            paddingLeft: '20px'
          }}>
            <li><strong>Server-Side Rendering:</strong> This page's HTML was generated on the server</li>
            <li><strong>Fast Initial Load:</strong> Content is visible immediately, no loading states</li>
            <li><strong>SEO Friendly:</strong> Search engines can crawl your content easily</li>
            <li><strong>Client Components:</strong> The like button above uses React hooks</li>
            <li><strong>Hybrid Architecture:</strong> Best of both worlds - SSR for static content, client for interactivity</li>
          </ul>
        </section>
      </main>

      <footer style={{
        marginTop: '80px',
        padding: '30px',
        background: '#f9f9f9',
        borderRadius: '12px',
        textAlign: 'center',
        color: '#666',
        border: '1px solid #e5e7eb'
      }}>
        <a
          href="/"
          style={{
            display: 'inline-block',
            padding: '12px 24px',
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            color: 'white',
            textDecoration: 'none',
            borderRadius: '8px',
            fontWeight: '600',
            fontSize: '1rem',
            transition: 'all 0.2s'
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = 'translateY(-2px)';
            e.currentTarget.style.boxShadow = '0 4px 12px rgba(102, 126, 234, 0.4)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = 'translateY(0)';
            e.currentTarget.style.boxShadow = 'none';
          }}
        >
          ← Back to Blog
        </a>
        <p style={{ marginTop: '20px', fontSize: '0.85rem', color: '#888' }}>
          Built with Virust - Server-Side Rendering for Rust
        </p>
      </footer>
    </div>
  );
}
