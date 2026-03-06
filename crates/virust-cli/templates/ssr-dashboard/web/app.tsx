import { StatsCard } from './components/StatsCard';
import { RevenueChart } from './components/RevenueChart';
import { UserTable } from './components/UserTable';
import { Stats, ChartData, User } from './types';

interface PageProps {
  stats: Stats;
  chartData: ChartData;
  users: User[];
}

export default function Dashboard({ stats, chartData, users }: PageProps) {
  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 py-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-3xl font-bold text-gray-900">Dashboard</h1>
              <p className="text-gray-600 mt-1">Welcome back! Here's what's happening.</p>
            </div>
            <div className="text-sm text-gray-500">
              Last updated: {new Date(stats.updated_at * 1000).toLocaleString()}
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8">
        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <StatsCard
            title="Total Users"
            value={stats.total_users.toLocaleString()}
            change="+12.5%"
            changeType="positive"
            icon="👥"
          />
          <StatsCard
            title="Active Sessions"
            value={stats.active_sessions.toString()}
            change="+5.2%"
            changeType="positive"
            icon="⚡"
          />
          <StatsCard
            title="Revenue"
            value={`$${stats.revenue.toLocaleString()}`}
            change="+18.3%"
            changeType="positive"
            icon="💰"
          />
          <StatsCard
            title="Conversion Rate"
            value={`${stats.conversion_rate}%`}
            change="-2.1%"
            changeType="negative"
            icon="📈"
          />
        </div>

        {/* Charts Section */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
          <RevenueChart data={chartData} />
          <div className="bg-white rounded-lg shadow p-6">
            <h3 className="text-lg font-semibold text-gray-900 mb-4">Recent Activity</h3>
            <div className="space-y-4">
              {users.slice(0, 5).map((user) => (
                <div key={user.id} className="flex items-center justify-between py-2 border-b border-gray-100 last:border-0">
                  <div>
                    <p className="font-medium text-gray-900">{user.name}</p>
                    <p className="text-sm text-gray-500">{user.email}</p>
                  </div>
                  <span className="px-2 py-1 text-xs rounded-full bg-blue-100 text-blue-700">
                    {user.role}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Users Table */}
        <UserTable users={users} />
      </main>
    </div>
  );
}
