#!/bin/bash
set -e
echo "========================================="
echo "RCSS Server Monitoring Setup"
echo "========================================="
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "Error: Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Navigate to monitoring directory
cd "$(dirname "$0")"

echo "Starting monitoring stack..."
docker-compose up -d

echo ""
echo "Waiting for services to start..."
sleep 10

# Check if services are running
echo ""
echo "Checking service status..."
docker-compose ps

echo ""
echo "========================================="
echo "Monitoring Stack Started Successfully!"
echo "========================================="
echo ""
echo "Access the following services:"
echo ""
echo "  Grafana:       http://localhost:3000"
echo "                 Username: admin"
echo "                 Password: admin"
echo ""
echo "  Prometheus:    http://localhost:9090"
echo "  Alertmanager:  http://localhost:9093"
echo ""
echo "RCSS Server metrics endpoint:"
echo "  http://localhost:55555/metrics"
echo ""
echo "To view logs:"
echo "  docker-compose logs -f"
echo ""
echo "To stop the monitoring stack:"
echo "  docker-compose down"
echo ""
echo "========================================="
