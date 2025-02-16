import { useEffect, useState } from 'react';
import SystemDashboard from './components/SystemDashboard.jsx';

function App() {
  const [systemData, setSystemData] = useState({
    processes: [],
    cpu: {},           // Changed from { usage: 0, cores: 0 }
    memory: {},        // Changed from { used: 0, total: 0 }
    disks: {},         // Changed from { used: 0, total: 0 }
    network: {}        // Changed from { rx_bytes: 0, tx_bytes: 0 }
});
  const [loading, setLoading] = useState(true);
  const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

  useEffect(() => {
    const fetchSystemData = async () => {
      try {
        setLoading(true);
        const response = await fetch(`${API_URL}/metrics`);
        const data = await response.json();
        
        // Only update if we got valid data
        if (data) {
          setSystemData(data);
        }
      } catch (error) {
        console.error('Error fetching system data:', error);
      } finally {
        setLoading(false);
      }
    };

    // Initial fetch
    fetchSystemData();

    // Set up polling
    const interval = setInterval(fetchSystemData, 30000);

    // Cleanup
    return () => clearInterval(interval);
  }, []);

  // Show loading or pass valid data
  return (
    <SystemDashboard 
      data={systemData}
      loading={loading}
    />
  );
}

export default App;