import { useEffect, useState } from 'react';
import SystemDashboard from './components/SystemDashboard.jsx';

function App() {
  const [systemData, setSystemData] = useState(null);
  const [isInitialLoading, setIsInitialLoading] = useState(true);
  const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

  useEffect(() => {
    const fetchSystemData = async () => {
      try {
        const response = await fetch(`${API_URL}/metrics`);
        const data = await response.json();
        setSystemData(data);
      } catch (error) {
        console.error('Error fetching system data:', error);
      } finally {
        setIsInitialLoading(false);
      }
    };

    // Initial fetch
    fetchSystemData();

    // Set up polling that won't cause flashing
    const interval = setInterval(async () => {
      try {
        const response = await fetch(`${API_URL}/metrics`);
        const newData = await response.json();
        // Smooth transition of data
        setSystemData(prevData => {
          if (!prevData) return newData;
          return {
            ...newData,
            // Preserve expanded state by keeping old processes array if new one is empty
            processes: newData.processes?.length ? newData.processes : prevData.processes,
          };
        });
      } catch (error) {
        console.error('Error fetching system data:', error);
      }
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  return (
    <SystemDashboard 
      data={systemData || {}}
      loading={isInitialLoading} // Only show loading on initial load
    />
  );
}

export default App;