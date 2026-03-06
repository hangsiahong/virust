import Link from 'next/link';
import { HomePageData } from './types';

interface PageProps {
  data: HomePageData;
}

export default function Home({ data }: PageProps) {
  return (
    <div className="min-h-screen bg-gradient-to-br from-indigo-50 via-white to-purple-50">
      {/* Navigation */}
      <nav className="bg-white/80 backdrop-blur-sm border-b border-gray-200 sticky top-0 z-50">
        <div className="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
          <h1 className="text-xl font-bold text-indigo-600">{{project_name}}</h1>
          <div className="flex gap-6">
            <Link href="/" className="text-gray-700 hover:text-indigo-600 transition">Home</Link>
            <Link href="/blog" className="text-gray-700 hover:text-indigo-600 transition">Blog</Link>
            <Link href="/dashboard" className="text-gray-700 hover:text-indigo-600 transition">Dashboard</Link>
            <Link href="/todos" className="text-gray-700 hover:text-indigo-600 transition">Todos</Link>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <div className="max-w-7xl mx-auto px-4 py-20 text-center">
        <h2 className="text-5xl font-bold text-gray-900 mb-6">{data.title}</h2>
        <p className="text-xl text-gray-600 mb-12">{data.subtitle}</p>

        {/* Features */}
        <div className="grid md:grid-cols-3 gap-8 mt-16">
          {data.features.map((feature, index) => (
            <div key={index} className="bg-white rounded-xl shadow-lg p-8 hover:shadow-xl transition">
              <div className="text-4xl mb-4">✨</div>
              <h3 className="text-xl font-semibold text-gray-900 mb-2">{feature}</h3>
              <p className="text-gray-600">
                Experience the power of modern web development with Virust
              </p>
            </div>
          ))}
        </div>

        {/* CTA */}
        <div className="mt-16">
          <Link
            href="/dashboard"
            className="inline-block bg-indigo-600 text-white px-8 py-4 rounded-lg font-semibold hover:bg-indigo-700 transition"
          >
            Get Started
          </Link>
        </div>
      </div>

      {/* Footer */}
      <footer className="bg-gray-900 text-white py-12 mt-20">
        <div className="max-w-7xl mx-auto px-4 text-center">
          <p className="text-gray-400">Built with Virust - SSG, ISR, SSR, and Caching</p>
        </div>
      </footer>
    </div>
  );
}
