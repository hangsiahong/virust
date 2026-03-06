import { BlogCard } from './components/BlogCard';
import { BlogPost } from './types';

interface PageProps {
  posts: BlogPost[];
}

export default function BlogHome({ posts }: PageProps) {
  return (
    <div className="min-h-screen bg-gradient-to-b from-gray-50 to-white">
      {/* Header */}
      <header className="bg-white border-b border-gray-200">
        <div className="max-w-6xl mx-auto px-4 py-6">
          <h1 className="text-4xl font-bold text-gray-900">My Blog</h1>
          <p className="text-gray-600 mt-2">Built with Virust, SSG, and ISR</p>
        </div>
      </header>

      {/* Blog Posts */}
      <main className="max-w-6xl mx-auto px-4 py-12">
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
          {posts.map((post) => (
            <BlogCard key={post.id} post={post} />
          ))}
        </div>
      </main>

      {/* Footer */}
      <footer className="bg-white border-t border-gray-200 mt-12">
        <div className="max-w-6xl mx-auto px-4 py-6 text-center text-gray-600">
          <p>© 2024 My Blog. Built with Virust.</p>
        </div>
      </footer>
    </div>
  );
}
