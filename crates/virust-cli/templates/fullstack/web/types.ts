export interface HomePageData {
  title: string;
  subtitle: string;
  features: string[];
}

export interface BlogPost {
  id: string;
  title: string;
  slug: string;
  excerpt: string;
  content: string;
  published_at: number;
}

export interface DashboardStats {
  total_users: number;
  active_sessions: number;
  revenue: number;
}

export interface Todo {
  id: string;
  title: string;
  description?: string;
  completed: boolean;
  created_at: number;
}
