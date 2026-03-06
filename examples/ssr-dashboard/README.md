# SSR Dashboard Example

A complete, working example of an analytics dashboard built with Virust, demonstrating server-side rendering with real-time data, charts, and interactive components.

## Features

- **Server-Side Rendering**: HTML generated on the server for fast page loads and excellent SEO
- **Real-Time Metrics**: Six key performance metrics with trend indicators
- **Interactive Charts**: Custom-built bar chart showing weekly traffic data
- **Activity Feed**: Recent activity stream with different event types
- **Client Components**: Interactive refresh button with React hooks
- **Modern UI**: Clean, responsive design with gradient accents and hover effects

## Project Structure

```
ssr-dashboard/
├── Cargo.toml              # Rust dependencies
├── README.md               # This file
├── src/
│   ├── main.rs             # Entry point
│   ├── lib.rs              # Library exports
│   └── api/
│       ├── mod.rs          # API module exports
│       └── route.rs        # Route handlers with dashboard data
└── web/
    ├── index.html          # HTML template with SSR placeholder
    ├── main.js             # Client-side JavaScript
    └── components/
        ├── Dashboard.jsx    # Main dashboard (server component)
        ├── StatCard.jsx     # Metric card (server component)
        ├── ActivityFeed.jsx # Activity stream (server component)
        ├── Chart.jsx        # Bar chart (server component)
        └── RefreshButton.jsx # Refresh button (client component)
```

## How It Works

### Data Flow

1. **Backend Fetching**: When a user visits `/`, the `dashboard()` function in `src/api/route.rs` is called
2. **Data Collection**: The function fetches metrics, chart data, and activities
3. **Props Passing**: Data is passed to the `Dashboard` component via `RenderedHtml::with_props()`
4. **Server Rendering**: All server components are rendered on the server using Bun
5. **HTML Injection**: The rendered HTML is injected into the `{{SSR_CONTENT}}` placeholder
6. **Client Hydration**: Client components (like `RefreshButton`) are hydrated in the browser

### Component Types

#### Server Components (Default)
- `Dashboard` - Main layout and data display
- `StatCard` - Metric cards with formatted values
- `ActivityFeed` - Activity stream with icons
- `Chart` - Custom bar chart visualization

Server components:
- Are rendered on the server
- Receive data from the backend as props
- Can't use React hooks
- Reduce JavaScript bundle size

#### Client Components (with 'use client')
- `RefreshButton` - Interactive refresh button

Client components:
- Are rendered in the browser
- Can use React hooks (useState, useEffect, etc.)
- Handle user interactions
- Provide interactivity

## Running the Example

### Prerequisites

- Rust installed
- Virust CLI installed
- Bun installed (for JSX rendering)

### Start the Server

```bash
cd examples/ssr-dashboard
virust dev
```

The server will start on `http://127.0.0.1:3000`

### View in Browser

1. Open `http://127.0.0.1:3000` to see the dashboard
2. Click the "Refresh Data" button to see client-side interactivity
3. Hover over chart bars to see values
4. View the page source to see the server-rendered HTML

## Key Files Explained

### `src/api/route.rs`

Contains the route handler that fetches data and initiates SSR:

```rust
#[get]
#[render_component("Dashboard")]
pub async fn dashboard() -> RenderedHtml {
    let metrics = get_dashboard_metrics();
    let chart_data = get_chart_data();
    let activities = get_recent_activities();

    let data = json!({
        "metrics": metrics,
        "chartData": chart_data,
        "activities": activities,
        "lastUpdated": /* timestamp */
    });

    RenderedHtml::with_props("Dashboard", data)
}
```

### `web/components/Dashboard.jsx`

Main server component that receives props and renders the layout:

```jsx
export default function Dashboard({ title, metrics, chartData, activities, lastUpdated }) {
    return (
        <div>
            <header>{title}</header>
            <StatCards metrics={metrics} />
            <Chart data={chartData} />
            <ActivityFeed activities={activities} />
        </div>
    );
}
```

### `web/components/Chart.jsx`

Server component that renders a custom bar chart:

```jsx
export default function Chart({ data }) {
    const maxValue = Math.max(...data.map(d => d.value));

    return (
        <div>
            {data.map(point => (
                <div style={{ height: `${(point.value / maxValue) * 100}%` }}>
                    {point.label}
                </div>
            ))}
        </div>
    );
}
```

### `web/components/RefreshButton.jsx`

Client component with React hooks:

```jsx
'use client';

import { useState } from 'react';

export default function RefreshButton() {
    const [isRefreshing, setIsRefreshing] = useState(false);

    const handleRefresh = () => {
        setIsRefreshing(true);
        // Simulate refresh
        setTimeout(() => setIsRefreshing(false), 1500);
    };

    return <button onClick={handleRefresh}>Refresh</button>;
}
```

## Customization

### Adding New Metrics

Edit the `get_dashboard_metrics()` function in `src/api/route.rs`:

```rust
MetricData {
    label: "New Metric".to_string(),
    value: 1234.0,
    change: 5.5,
    trend: "up".to_string(),
}
```

### Updating Chart Data

Modify the `get_chart_data()` function:

```rust
ChartPoint {
    label: "Day".to_string(),
    value: 1500.0,
}
```

### Changing the UI

Modify the JSX files in `web/components/`:
- `Dashboard.jsx` - Overall layout and sections
- `StatCard.jsx` - Metric card appearance
- `Chart.jsx` - Chart visualization
- `ActivityFeed.jsx` - Activity stream layout
- `RefreshButton.jsx` - Button appearance and behavior

### Styling

The components use inline styles for simplicity. For production, consider:
- CSS modules
- Tailwind CSS
- Styled-components
- Or any other React styling solution

## Architecture Benefits

### Performance
- **Fast Initial Load**: HTML is generated on the server
- **Reduced Bundle Size**: Server components don't send JavaScript to the client
- **Excellent SEO**: Search engines can crawl all content

### Developer Experience
- **Type Safety**: Rust backend ensures data integrity
- **Separation of Concerns**: Server and client components have clear roles
- **Flexibility**: Mix SSR and CSR as needed

### User Experience
- **Instant Content**: No loading states for initial page load
- **Progressive Enhancement**: Core content works without JavaScript
- **Interactivity**: Client components provide dynamic features

## Learning Resources

- [Virust SSR Guide](../../docs/ssr-guide.md) - Complete SSR documentation
- [Blog Example](../ssr-blog/) - Simpler SSR example
- [Templates vs Examples](../../docs/ssr-guide.md#templates-vs-examples) - Understanding the difference

## Next Steps

1. **Explore the code**: Read through the files to understand the data flow
2. **Make changes**: Modify metrics, add charts, or change the layout
3. **Build from scratch**: Use `virust init my-dashboard --template ssr-dashboard`
4. **Add features**: Implement real-time updates, filters, or export functionality

## Advanced Features to Try

- **Real-Time Updates**: Use WebSockets to push new data to the dashboard
- **Data Persistence**: Connect to a real database
- **Authentication**: Add user login and personalized dashboards
- **Export**: Add CSV/PDF export functionality
- **Dark Mode**: Implement theme switching

## License

This example is part of the Virust project and is available under the same license.
