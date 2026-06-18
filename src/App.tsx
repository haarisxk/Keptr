import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

export default function App() {
  const [isInitialized, setIsInitialized] = useState<boolean | null>(null);
  const [isLocked, setIsLocked] = useState(true);
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    // Check initial vault status
    invoke<{ is_initialized: boolean; is_unlocked: boolean }>('check_vault_status')
      .then((status) => {
        setIsInitialized(status.is_initialized);
        setIsLocked(!status.is_unlocked);
      })
      .catch((e) => setError(String(e)));
  }, []);

  const handleInitialize = async () => {
    try {
      setError('');
      await invoke('initialize_vault', { password });
      setIsInitialized(true);
      setIsLocked(false);
      setPassword('');
    } catch (e) {
      setError(String(e));
    }
  };

  const handleUnlock = async () => {
    try {
      setError('');
      await invoke('unlock_vault', { password });
      setIsLocked(false);
      setPassword('');
    } catch (e) {
      setError(String(e));
    }
  };

  const handleLock = async () => {
    try {
      await invoke('lock_vault');
      setIsLocked(true);
    } catch (e) {
      console.error(e);
    }
  };

  if (isInitialized === null) {
    return <div className="min-h-screen flex items-center justify-center bg-gray-50">Loading...</div>;
  }

  if (!isInitialized) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
        <div className="bg-white rounded-xl shadow-sm border border-gray-100 p-8 w-full max-w-sm space-y-6">
          <div className="text-center">
            <h1 className="text-2xl font-semibold text-gray-900 mb-1">Welcome to Keptr</h1>
            <p className="text-sm text-gray-500">Create a master password to initialize your secure vault.</p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1" htmlFor="master-password">
                Master Password
              </label>
              <input
                id="master-password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                placeholder="Create master password"
              />
            </div>
            {error && <p className="text-sm text-red-500">{error}</p>}
            <button
              onClick={handleInitialize}
              className="w-full bg-indigo-500 text-white px-4 py-2.5 rounded-lg text-sm font-medium hover:bg-indigo-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 transition-colors"
            >
              Create Vault
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (isLocked) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 p-4">
        <div className="bg-white rounded-xl shadow-sm border border-gray-100 p-8 w-full max-w-sm space-y-6">
          <div className="text-center">
            <h1 className="text-2xl font-semibold text-gray-900 mb-1">Keptr</h1>
            <p className="text-sm text-gray-500">Your Digital Life, Kept Secure.</p>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1" htmlFor="master-password">
                Master Password
              </label>
              <input
                id="master-password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && handleUnlock()}
                className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                placeholder="Enter master password"
              />
            </div>
            {error && <p className="text-sm text-red-500">{error}</p>}
            <button
              onClick={handleUnlock}
              className="w-full bg-indigo-500 text-white px-4 py-2.5 rounded-lg text-sm font-medium hover:bg-indigo-600 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 transition-colors"
            >
              Unlock Vault
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex bg-gray-50">
      <div className="w-64 bg-white border-r border-gray-200 p-4">
        <h2 className="text-lg font-semibold text-gray-900 mb-6">Keptr</h2>
        <nav className="space-y-1 text-sm">
          <a href="#" className="block px-3 py-2 bg-indigo-500 text-white rounded-lg">All Items</a>
          <a href="#" className="block px-3 py-2 text-gray-600 hover:bg-gray-100 rounded-lg">Logins</a>
          <a href="#" className="block px-3 py-2 text-gray-600 hover:bg-gray-100 rounded-lg">Secure Notes</a>
          <a href="#" className="block px-3 py-2 text-gray-600 hover:bg-gray-100 rounded-lg">Passkeys</a>
        </nav>
      </div>

      <div className="flex-1 p-8">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-semibold text-gray-900">Vault</h1>
          <button
            onClick={handleLock}
            className="px-4 py-2 text-sm text-gray-600 hover:bg-gray-200 rounded-lg transition-colors"
          >
            Lock Vault
          </button>
        </div>

        <div className="bg-white rounded-xl shadow-sm border border-gray-100 text-center p-12">
          <p className="text-gray-500 mb-4">Your vault is empty.</p>
          <button className="bg-indigo-500 text-white px-4 py-2.5 rounded-lg text-sm font-medium hover:bg-indigo-600 transition-colors">
            Add Item
          </button>
        </div>
      </div>
    </div>
  );
}
