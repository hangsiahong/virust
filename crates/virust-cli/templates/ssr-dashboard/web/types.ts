export interface Stats {
  total_users: number;
  active_sessions: number;
  revenue: number;
  conversion_rate: number;
  updated_at: number;
}

export interface ChartData {
  labels: string[];
  values: number[];
}

export interface User {
  id: string;
  name: string;
  email: string;
  role: string;
  created_at: number;
}
