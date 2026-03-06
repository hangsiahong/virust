import { ChartData } from '../types';

interface RevenueChartProps {
  data: ChartData;
}

export function RevenueChart({ data }: RevenueChartProps) {
  const maxValue = Math.max(...data.values);
  const chartHeight = 200;

  return (
    <div className="bg-white rounded-lg shadow-md p-6">
      <h3 className="text-lg font-semibold text-gray-900 mb-4">Revenue (Last 7 Days)</h3>
      <div className="relative" style={{ height: `${chartHeight}px` }}>
        {/* Y-axis labels */}
        <div className="absolute left-0 top-0 bottom-0 w-12 flex flex-col justify-between text-xs text-gray-500">
          <span>${maxValue}</span>
          <span>${Math.round(maxValue / 2)}</span>
          <span>0</span>
        </div>

        {/* Chart area */}
        <div className="ml-16 h-full flex items-end justify-between gap-2">
          {data.values.map((value, index) => {
            const height = (value / maxValue) * chartHeight;
            return (
              <div key={index} className="flex-1 flex flex-col items-center">
                <div
                  className="w-full bg-blue-500 rounded-t hover:bg-blue-600 transition-colors"
                  style={{ height: `${height}px` }}
                  title={`$${value}`}
                />
                <span className="text-xs text-gray-500 mt-2 transform -rotate-45 origin-top-left">
                  {data.labels[index].split('-').slice(1).join('/')}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
