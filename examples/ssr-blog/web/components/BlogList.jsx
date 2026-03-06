// BlogList.jsx - Server-side rendered blog list
import LikeButton from './LikeButton';

export default function BlogList({ posts }) {
  return (
    <div style={{
      maxWidth: '900px',
      margin: '0 auto',
      padding: '40px 20px',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      <header style={{
        marginBottom: '60px',
        paddingBottom: '30px',
        borderBottom: '3px solid #eaeaea'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '15px' }}>
          <div style={{
            width: '60px',
            height: '60px',
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            borderRadius: '12px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: '28px',
            color: 'white',
            fontWeight: 'bold'
          }}>
            V
          </div>
          <div>
            <h1 style={{
              fontSize: '2.5rem',
              marginBottom: '8px',
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              WebkitBackgroundClip: 'text',
              WebkitTextFillColor: 'transparent',
              backgroundClip: 'text'
            }}>
              Virust Blog
            </h1>
            <p style={{ color: '#666', fontSize: '1.1rem' }}>
              Server-Side Rendering with Rust & React
            </p>
          </div>
        </div>
      </header>

      <main>
        <section style={{ marginBottom: '50px' }}>
          <div style={{
            padding: '30px',
            background: 'linear-gradient(135deg, #667eea15 0%, #764ba215 100%)',
            borderRadius: '16px',
            marginBottom: '40px',
            border: '2px solid #667eea30'
          }}>
            <h2 style={{ fontSize: '1.8rem', marginBottom: '15px', color: '#333' }}>
              Welcome to the SSR Blog Example
            </h2>
            <p style={{ color: '#555', lineHeight: '1.7', fontSize: '1.05rem' }}>
              This blog demonstrates server-side rendering with Virust. The HTML is generated on the server
              and sent to your browser, providing fast page loads and excellent SEO. The like button below
              is a client component that uses React hooks.
            </p>
            <div style={{ marginTop: '20px' }}>
              <LikeButton initialLikes={125} />
            </div>
          </div>

          <h2 style={{ fontSize: '2rem', marginBottom: '30px', color: '#333' }}>
            Latest Posts
          </h2>

          {posts && posts.length > 0 ? (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '25px' }}>
              {posts.map((post) => (
                <article
                  key={post.id}
                  style={{
                    padding: '30px',
                    background: 'white',
                    borderRadius: '12px',
                    boxShadow: '0 2px 8px rgba(0,0,0,0.08)',
                    border: '1px solid #e5e7eb',
                    transition: 'all 0.2s',
                    cursor: 'pointer'
                  }}
                  onClick={() => window.location.href = '/post'}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.transform = 'translateY(-2px)';
                    e.currentTarget.style.boxShadow = '0 4px 16px rgba(0,0,0,0.12)';
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.transform = 'translateY(0)';
                    e.currentTarget.style.boxShadow = '0 2px 8px rgba(0,0,0,0.08)';
                  }}
                >
                  <div style={{ marginBottom: '15px' }}>
                    <span style={{
                      padding: '6px 12px',
                      background: 'linear-gradient(135deg, #667eea20 0%, #764ba220 100%)',
                      color: '#667eea',
                      borderRadius: '20px',
                      fontSize: '0.85rem',
                      fontWeight: '600',
                      textTransform: 'uppercase',
                      letterSpacing: '0.5px'
                    }}>
                      Blog Post
                    </span>
                  </div>

                  <h3 style={{
                    fontSize: '1.6rem',
                    marginBottom: '12px',
                    color: '#1a202c',
                    fontWeight: '700'
                  }}>
                    {post.title}
                  </h3>

                  <p style={{
                    color: '#666',
                    lineHeight: '1.7',
                    marginBottom: '20px',
                    fontSize: '1.05rem'
                  }}>
                    {post.excerpt}
                  </p>

                  <div style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    paddingTop: '15px',
                    borderTop: '1px solid #e5e7eb'
                  }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                      <div style={{
                        width: '32px',
                        height: '32px',
                        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                        borderRadius: '50%',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: 'white',
                        fontSize: '0.85rem',
                        fontWeight: 'bold'
                      }}>
                        {post.author.charAt(0)}
                      </div>
                      <div>
                        <div style={{ fontSize: '0.9rem', fontWeight: '600', color: '#333' }}>
                          {post.author}
                        </div>
                        <div style={{ fontSize: '0.8rem', color: '#888' }}>
                          {post.date}
                        </div>
                      </div>
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: '20px' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                        <span style={{ fontSize: '1.2rem' }}>♥</span>
                        <span style={{ fontSize: '0.9rem', fontWeight: '600', color: '#666' }}>
                          {post.likes}
                        </span>
                      </div>
                      <span style={{
                        color: '#667eea',
                        fontSize: '0.9rem',
                        fontWeight: '600',
                        textTransform: 'uppercase',
                        letterSpacing: '0.5px'
                      }}>
                        Read More →
                      </span>
                    </div>
                  </div>
                </article>
              ))}
            </div>
          ) : (
            <div style={{
              padding: '40px',
              background: '#f9f9f9',
              borderRadius: '8px',
              textAlign: 'center',
              color: '#666'
            }}>
              <p style={{ fontSize: '1.1rem' }}>No posts available yet. Check back soon!</p>
            </div>
          )}
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
        <p style={{ marginBottom: '10px', fontSize: '0.95rem' }}>
          Built with <strong>Virust</strong> - Server-Side Rendering for Rust
        </p>
        <p style={{ fontSize: '0.85rem', color: '#888' }}>
          View the source code to learn how SSR works
        </p>
      </footer>
    </div>
  );
}
