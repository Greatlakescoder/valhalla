import { useEffect, useState } from 'react';
import ProcessMonitor from './components/ProcessMonitor.jsx';

function App() {
  const [processes, setProcesses] = useState([]); // Initialize with empty array
  const [loading, setLoading] = useState(true);  // Add loading state
  const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';
  useEffect(() => {
    const fetchProcesses = async () => {
      try {
        setLoading(true);  // Set loading before fetch
        const response = await fetch(`${API_URL}/processes`);
        const data = await response.json();
        // Only set the processes if we got valid data
        if (data && data[0]) {
          setProcesses(data[0]);
        }
      } catch (error) {
        console.error('Error fetching processes:', error);
      } finally {
        setLoading(false);  // Always turn off loading
      }
    };

    // Initial fetch
    fetchProcesses();

    // Set up polling
    const interval = setInterval(fetchProcesses, 5000);

    // Cleanup
    return () => clearInterval(interval);
  }, []);

  // Show loading or pass valid data
  return (
    <ProcessMonitor 
      processes={processes} 
      loading={loading}
    />
  );
}

export default App;