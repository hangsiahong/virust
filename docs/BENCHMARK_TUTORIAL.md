# Performance Benchmark Tutorial: Virust vs Next.js

**Learn how to run your own benchmarks and reproduce the performance comparison results.**

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Setup](#setup)
3. [Running the Benchmarks](#running-the-benchmarks)
4. [Understanding Results](#understanding-results)
5. [Customizing Tests](#customizing-tests)
6. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required Software

1. **Rust toolchain**
   ```bash
   # Check if installed
   rustc --version
   cargo --version

   # If not, install from:
   # https://www.rust-lang.org/tools/install
   ```

2. **Node.js and npm**
   ```bash
   node --version  # Should be v18+
   npm --version
   ```

3. **Benchmark tool**
   ```bash
   npm install -g autocannon
   ```

### System Requirements

- **OS:** Linux, macOS, or Windows (WSL2 recommended for Windows)
- **RAM:** 4GB minimum, 8GB+ recommended
- **Disk:** 1GB free space
- **CPU:** Any modern multi-core processor

---

## Setup

### Step 1: Clone or Create Test Apps

You can either use the pre-created benchmark apps or create your own.

#### Option A: Use Pre-Created Apps (Recommended)

```bash
# Create benchmark directory
mkdir -p ~/benchmark-tutorial
cd ~/benchmark-tutorial

# Download or copy the benchmark apps
# (These would be in the repo or created via virust init)
```

#### Option B: Create From Scratch

**Create Virust (Axum) App:**

```bash
cd ~/benchmark-tutorial

# Create directory
mkdir virust-app && cd virust-app

# Initialize Rust project
cargo init --name virust-bench

# Update Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "virust-bench"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
EOF

# Create main.rs
cat > src/main.rs << 'EOF'
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::time::SystemTime;

#[derive(Serialize)]
struct PingResponse {
    message: String,
    timestamp: u64,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ping", get(ping_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .expect("Failed to bind");

    println!("✓ Virust server running on http://127.0.0.1:3001");
    println!("   Ping endpoint: http://127.0.0.1:3001/ping");

    axum::serve(listener, app).await.expect("Server error");
}

async fn ping_handler() -> Json<PingResponse> {
    Json(PingResponse {
        message: "PONG".to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}
EOF

# Build in release mode
cargo build --release

echo "✓ Virust app created and built"
```

**Create Next.js App:**

```bash
cd ~/benchmark-tutorial

# Create Next.js app
npx create-next-app@latest nextjs-app --typescript --tailwind --yes

# Or create manually:
mkdir nextjs-app && cd nextjs-app

# Initialize package.json
cat > package.json << 'EOF'
{
  "name": "nextjs-bench",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start"
  },
  "dependencies": {
    "next": "^15.1.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }
}
EOF

# Install dependencies
npm install

# Create API route
mkdir -p src/app/api/ping
cat > src/app/api/ping/route.ts << 'EOF'
import { NextResponse } from 'next/server';

export async function GET() {
  return NextResponse.json({
    message: 'PONG',
    timestamp: Math.floor(Date.now() / 1000)
  });
}
EOF

# Create simple page
cat > src/app/page.tsx << 'EOF'
export default function Home() {
  return <div>Benchmark App</div>;
}
EOF

# Create layout
cat > src/app/layout.tsx << 'EOF'
export const metadata = {
  title: 'Benchmark',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
EOF

# Build
npm run build

echo "✓ Next.js app created and built"
```

### Step 2: Verify Both Apps Work

**Test Virust:**

```bash
cd ~/benchmark-tutorial/virust-app

# Start server
./target/release/virust-bench &
VIRUST_PID=$!

# Wait a moment
sleep 2

# Test endpoint
curl http://localhost:3001/ping
# Expected output: {"message":"PONG","timestamp":<number>}

# Stop server
kill $VIRUST_PID
```

**Test Next.js:**

```bash
cd ~/benchmark-tutorial/nextjs-app

# Start server
PORT=3001 npm start &
NEXT_PID=$!

# Wait for startup (can take 5-10 seconds)
sleep 8

# Test endpoint
curl http://localhost:3001/api/ping
# Expected output: {"message":"PONG","timestamp":<number>}

# Stop server
kill $NEXT_PID
```

---

## Running the Benchmarks

### Quick Start (Automated Script)

Create this benchmark script:

```bash
#!/bin/bash
# File: benchmark.sh

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PORT=3001

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Virust vs Next.js Benchmark${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Kill any existing processes
fuser -k ${PORT}/tcp 2>/dev/null || true
sleep 2

# ============================================
# VIRUST BENCHMARK
# ============================================
echo -e "${YELLOW}[1/4] Benchmarking Virust (Axum)...${NC}\n"

cd ~/benchmark-tutorial/virust-app

# Cold start
echo "Measuring cold start..."
START=$(date +%s%N)
./target/release/virust-bench > /tmp/virust-bench.log 2>&1 &
PID=$!

# Wait for ready
while ! curl -s http://localhost:${PORT}/ping > /dev/null 2>&1; do
  sleep 0.01
done
END=$(date +%s%N)
COLD_START=$((($END - $START) / 1000000))
echo -e "${GREEN}✓${NC} Cold start: ${COLD_START}ms"

# Load test
sleep 2
echo "Running load test (10 connections, 10 seconds)..."
RESULT=$(autocannon -c 10 -d 10 http://localhost:${PORT}/ping 2>&1)

echo "$RESULT" | grep -E "Req/Sec|Latency|2.5%|50%|97.5%|99%"

# Memory
sleep 2
MEMORY=$(ps -o rss= -p $PID | awk '{printf "%.1f", $1/1024}')
echo -e "${GREEN}✓${NC} Memory: ${MEMORY}MB"

# Binary size
SIZE=$(du -h target/release/virust-bench | awk '{print $1}')
echo -e "${GREEN}✓${NC} Binary size: ${SIZE}"

# Cleanup
kill $PID 2>/dev/null || true
fuser -k ${PORT}/tcp 2>/dev/null || true
sleep 2

# ============================================
# NEXT.JS BENCHMARK
# ============================================
echo ""
echo -e "${YELLOW}[2/4] Benchmarking Next.js...${NC}\n"

cd ~/benchmark-tutorial/nextjs-app

# Cold start
echo "Measuring cold start..."
START=$(date +%s%N)
PORT=${PORT} npm start > /tmp/nextjs-bench.log 2>&1 &
PID=$!

# Wait for ready (Next.js takes longer)
while ! curl -s http://localhost:${PORT}/api/ping > /dev/null 2>&1; do
  sleep 0.01
done
END=$(date +%s%N)
COLD_START=$((($END - $START) / 1000000))
echo -e "${GREEN}✓${NC} Cold start: ${COLD_START}ms"

# Load test
sleep 5
echo "Running load test (10 connections, 10 seconds)..."
RESULT=$(autocannon -c 10 -d 10 http://localhost:${PORT}/api/ping 2>&1)

echo "$RESULT" | grep -E "Req/Sec|Latency|2.5%|50%|97.5%|99%"

# Memory
sleep 2
MEMORY=$(ps -o rss= -p $PID 2>/dev/null | awk '{printf "%.1f", $1/1024}' || echo "N/A")
echo -e "${GREEN}✓${NC} Memory: ${MEMORY}MB"

# Node modules size
SIZE=$(du -sh node_modules 2>/dev/null | awk '{print $1}')
echo -e "${GREEN}✓${NC} node_modules: ${SIZE}"

# Cleanup
kill $PID 2>/dev/null || true
fuser -k ${PORT}/tcp 2>/dev/null || true

echo ""
echo -e "${GREEN}✓ Benchmark complete!${NC}"
```

Make it executable and run:

```bash
chmod +x benchmark.sh
./benchmark.sh
```

### Manual Step-by-Step

For more control, run benchmarks manually:

#### **1. Virust Benchmark**

```bash
cd ~/benchmark-tutorial/virust-app

# Start server
./target/release/virust-bench &
VIRUST_PID=$!

# Wait 2 seconds for startup
sleep 2

# Verify it's working
curl http://localhost:3001/ping

# Run 10-second load test with 10 concurrent connections
autocannon -c 10 -d 10 http://localhost:3001/ping

# Check memory usage
ps -o rss= -p $VIRUST_PID | awk '{print "Memory:", $1/1024 "MB"}'

# Get PID for later cleanup
echo $VIRUST_PID

# Stop server when done
kill $VIRUST_PID
```

#### **2. Next.js Benchmark**

```bash
cd ~/benchmark-tutorial/nextjs-app

# Start server on port 3001
PORT=3001 npm start &
NEXT_PID=$!

# Wait 8-10 seconds for startup (Next.js takes longer)
sleep 8

# Verify it's working
curl http://localhost:3001/api/ping

# Run 10-second load test with 10 concurrent connections
autocannon -c 10 -d 10 http://localhost:3001/api/ping

# Check memory usage
ps -o rss= -p $NEXT_PID | awk '{print "Memory:", $1/1024 "MB"}'

# Get PID for later cleanup
echo $NEXT_PID

# Stop server when done
kill $NEXT_PID
```

---

## Understanding Results

### Reading autocannon Output

```
┌─────────┬──────┬──────┬───────┬───────┬─────────┬─────────┬──────┐
│ Stat    │ 2.5% │ 50%  │ 97.5% │ 99%   │ Avg     │ Stdev   │ Max  │
├─────────┼──────┼──────┼───────┼───────┼─────────┼─────────┼──────┤
│ Latency │ 0 ms │ 0 ms │ 0 ms  │ 0 ms  │ 0.01 ms │ 0.04 ms │ 9 ms │
└─────────┴──────┴──────┴───────┴───────┴─────────┴─────────┴──────┘
```

**What this means:**
- **2.5%, 50%, 97.5%, 99%:** Percentiles (97.5% of requests took ≤ 0ms)
- **Avg:** Average response time
- **Stdev:** Standard deviation (consistency)
- **Max:** Worst response time

```
┌───────────┬─────────┬─────────┬─────────┬─────────┬──────────┬──────────┬─────────┐
│ Stat      │ 1%      │ 2.5%    │ 50%     │ 97.5%   │ Avg       │ Stdev    │ Min     │
├───────────┼─────────┼─────────┼─────────┼─────────┼───────────┼──────────┼─────────┤
│ Req/Sec   │ 96,895  │ 96,895  │ 116,031 │ 128,127 │ 112,505.6 │ 10,819.8 │ 96,860  │
└───────────┴─────────┴─────────┴─────────┴─────────┴───────────┴──────────┴─────────┘
```

**What this means:**
- **Req/Sec:** Requests per second (average)
- **Min:** Minimum requests per second seen
- **Max:** Peak requests per second achieved

### Key Metrics to Track

1. **Cold Start Time** - Time from process start to first response
   - Critical for: Serverless functions, auto-scaling
   - Good: <100ms | Excellent: <50ms

2. **Throughput (Req/Sec)** - Requests handled per second
   - Critical for: High-traffic APIs
   - Good: >10k/sec | Excellent: >50k/sec

3. **Average Latency** - Mean response time
   - Critical for: User experience
   - Good: <10ms | Excellent: <1ms

4. **P99 Latency** - 99th percentile (worst 1%)
   - Critical for: SLA compliance
   - Good: <100ms | Excellent: <10ms

5. **Memory Usage** - RAM consumed
   - Critical for: Container/Docker efficiency
   - Good: <100MB | Excellent: <20MB

---

## Customizing Tests

### Change Endpoint

Modify the endpoint in each app to test your actual API:

**Virust (`src/main.rs`):**
```rust
async fn ping_handler() -> Json<PingResponse> {
    Json(PingResponse {
        message: "CUSTOM_RESPONSE".to_string(),
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}
```

**Next.js (`src/app/api/ping/route.ts`):**
```typescript
export async function GET() {
  return NextResponse.json({
    message: 'CUSTOM_RESPONSE',
    timestamp: Math.floor(Date.now() / 1000)
  });
}
```

### Change Load Parameters

```bash
# Lighter load (5 connections, 5 seconds)
autocannon -c 5 -d 5 http://localhost:3001/ping

# Heavier load (50 connections, 30 seconds)
autocannon -c 50 -d 30 http://localhost:3001/ping

# Single connection test
autocannon -c 1 -d 5 http://localhost:3001/ping
```

### Test Different Scenarios

**Database Query Simulation:**

Add database query to your endpoint:

```rust
// Virust
async fn db_handler() -> Json<Vec<User>> {
    let users = sqlx::query_as::<User>("SELECT * FROM users LIMIT 100")
        .fetch_all(pool)
        .await
        .unwrap();
    Json(users)
}
```

```typescript
// Next.js
export async function GET() {
  const users = await db.query('SELECT * FROM users LIMIT 100');
  return NextResponse.json(users);
}
```

**Complex Computation:**

```rust
// Virust
async fn compute_handler() -> Json<Result> {
    let result = complex_computation().await;
    Json(result)
}
```

```typescript
// Next.js
export async function GET() {
  const result = await complexComputation();
  return NextResponse.json(result);
}
```

---

## Advanced Benchmarking

### Measure Percentile Latencies

Get detailed latency breakdown:

```bash
autocannon -c 10 -d 10 http://localhost:3001/ping | grep "Latency"
```

### Test Different Connection Counts

```bash
# Compare performance across different loads
for conn in 1 5 10 50 100; do
  echo "Testing with $conn connections..."
  autocannon -c $conn -d 5 http://localhost:3001/ping | grep "Req/Sec"
  sleep 2
done
```

### Generate HTML Report

```bash
autocannon -c 10 -d 10 http://localhost:3001/ping --format json > results.json

# Convert to readable format
jq . results.json
```

---

## Troubleshooting

### Port Already in Use

**Error:** `Error: listen EADDRINUSE :3001`

**Solution:**
```bash
# Find process using port 3001
lsof -ti:3001

# Kill it
kill -9 <PID>

# Or use fuser to kill all
fuser -k 3001/tcp
```

### Next.js Won't Start

**Possible issues:**

1. **Not built yet:**
   ```bash
   npm run build
   ```

2. **Port conflict:**
   ```bash
   PORT=3002 npm start
   # Then test on port 3002
   curl http://localhost:3002/api/ping
   ```

3. **Missing dependencies:**
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   npm run build
   ```

### Virust Won't Compile

**Common issues:**

1. **Missing Rust:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Dependencies not installing:**
   ```bash
   cargo clean
   cargo build --release
   ```

3. **Port binding permission denied:**
   ```bash
   # Use a different port
   # Edit main.rs: bind("127.0.0.1:3002")
   ```

### Autocannon Not Installed

**Install it:**
```bash
npm install -g autocannon

# Or use npx
npx autocannon -c 10 -d 5 http://localhost:3001/ping
```

### Inconsistent Results

**For consistent results:**

1. **Close other applications**
   ```bash
   # Close browser, IDE, etc.
   ```

2. **Run multiple times**
   ```bash
   for i in {1..3}; do
     autocannon -c 10 -d 10 http://localhost:3001/ping
   done
   ```

3. **Use warmup period**
   ```bash
   # Make a few requests first
   curl http://localhost:3001/ping
   curl http://localhost:3001/ping

   # Then run benchmark
   autocannon -c 10 -d 10 http://localhost:3001/ping
   ```

---

## Best Practices

### 1. Run Multiple Iterations

```bash
for i in 1 2 3; do
  echo "=== Run $i ==="
  ./benchmark.sh 2>&1 | tee results-run-$i.txt
  sleep 5
done
```

### 2. Compare Different Implementations

Create multiple endpoints and test each:

```bash
# Test simple endpoint
autocannon -c 10 -d 5 http://localhost:3001/ping

# Test complex endpoint
autocannon -c 10 -d 5 http://localhost:3001/api/users

# Test database endpoint
autocannon -c 10 -d 5 http://localhost:3001/api/posts
```

### 3. Document Your Environment

```bash
echo "Benchmark Environment:" > environment.txt
echo "Date: $(date)" >> environment.txt
echo "CPU: $(lscpu | grep 'Model name' | head -1)" >> environment.txt
echo "RAM: $(free -h | grep Mem | awk '{print $2}')" >> environment.txt
echo "Node.js: $(node --version)" >> environment.txt
echo "Rust: $(rustc --version | awk '{print $2}')" >> environment.txt
echo "OS: $(uname -a)" >> environment.txt
```

### 4. Save Raw Results

```bash
# Save full autocannon output
autocannon -c 10 -d 10 http://localhost:3001/ping > virust-raw.txt

# Save as JSON for analysis
autocannon -c 10 -d 10 http://localhost:3001/ping --format json > virust-results.json
```

---

## Example Workflow

Complete workflow for running and documenting benchmarks:

```bash
#!/bin/bash
# File: full-benchmark.sh

echo "Starting comprehensive benchmark..."
echo ""

# Environment info
echo "=== ENVIRONMENT ===" > report.txt
echo "Date: $(date)" >> report.txt
echo "CPU: $(lscpu | grep 'Model name' | head -1)" >> report.txt
echo "RAM: $(free -h | grep Mem | awk '{print $2}')" >> report.txt
echo "" >> report.txt

# Virust benchmark
cd ~/benchmark-tutorial/virust-app
./target/release/virust-bench &
VIRUST_PID=$!
sleep 2
echo "=== VIRUST ===" >> report.txt
autocannon -c 10 -d 10 http://localhost:3001/ping >> report.txt
ps -o rss= -p $VIRUST_PID | awk '{print "Memory:", $1/1024 "MB"}' >> report.txt
kill $VIRUST_PID

# Next.js benchmark
cd ~/benchmark-tutorial/nextjs-app
PORT=3001 npm start &
NEXT_PID=$!
sleep 8
echo "" >> report.txt
echo "=== NEXT.JS ===" >> report.txt
autocannon -c 10 -d 10 http://localhost:3001/api/ping >> report.txt
ps -o rss= -p $NEXT_PID | awk '{print "Memory:", $1/1024 "MB"}' >> report.txt
kill $NEXT_PID

echo ""
echo "Benchmark complete! Results saved to report.txt"
cat report.txt
```

---

## Next Steps

1. **Run the tutorial** - Follow the steps to create and benchmark both apps
2. **Customize the tests** - Modify endpoints to match your use case
3. **Compare results** - Document your findings
4. **Share results** - Include environment details for reproducibility

---

## Getting Help

If you encounter issues:

1. **Check the logs:**
   ```bash
   cat /tmp/virust-bench.log
   cat /tmp/nextjs-bench.log
   ```

2. **Verify servers are running:**
   ```bash
   curl http://localhost:3001/ping
   curl http://localhost:3001/api/ping
   ```

3. **Check processes:**
   ```bash
   ps aux | grep -E "virust|next"
   ```

4. **See example results:**
   - `docs/VIRUST_VS_NEXTJS_REAL_BENCHMARK.md` - Real measured results
   - `docs/VIRUST_VS_NEXTJS.md` - Theoretical comparison (now validated)

---

## Summary

This tutorial gives you everything you need to:

✅ Create benchmark apps from scratch
✅ Run performance tests
✅ Measure cold starts, throughput, latency, memory
✅ Understand autocannon output
✅ Customize tests for your use case
✅ Troubleshoot common issues

**Remember:** Results vary based on hardware, OS, and system load. Always document your environment and run multiple iterations for consistent results!

---

**Last Updated:** March 6, 2026
**Tested On:** AMD Ryzen AI 9 HX 370, Linux x86_64, Node.js v24.14.0, Rust 1.93.1
