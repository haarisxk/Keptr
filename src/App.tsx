import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface LoginItemDto {
  id: string;
  name: string;
  url: string;
  username: string;
  password_str: string;
  notes: string;
}

export default function App() {
  const [isInitialized, setIsInitialized] = useState<boolean | null>(null);
  const [isLocked, setIsLocked] = useState(true);
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');

  // Vault state
  const [items, setItems] = useState<LoginItemDto[]>([]);
  const [isAddingItem, setIsAddingItem] = useState(false);
  const [newItem, setNewItem] = useState({
    name: '',
    url: '',
    username: '',
    password_str: '',
    notes: '',
  });

  useEffect(() => {
    // Check initial vault status
    invoke<{ is_initialized: boolean; is_unlocked: boolean }>('check_vault_status')
      .then((status) => {
        setIsInitialized(status.is_initialized);
        setIsLocked(!status.is_unlocked);
      })
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    if (!isLocked && isInitialized) {
      loadItems();
    }
  }, [isLocked, isInitialized]);

  const loadItems = async () => {
    try {
      const fetchedItems = await invoke<LoginItemDto[]>('get_items');
      setItems(fetchedItems);
    } catch (e) {
      console.error('Failed to load items:', e);
    }
  };

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
      setItems([]);
    } catch (e) {
      console.error(e);
    }
  };

  const handleAddItem = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await invoke('add_item', { payload: newItem });
      setIsAddingItem(false);
      setNewItem({ name: '', url: '', username: '', password_str: '', notes: '' });
      loadItems();
    } catch (e) {
      console.error('Failed to add item:', e);
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
    <div className="min-h-screen flex bg-gray-50 relative">
      <div className="w-64 bg-white border-r border-gray-200 p-4 flex flex-col">
        <h2 className="text-lg font-semibold text-gray-900 mb-6 px-3">Keptr</h2>
        <nav className="space-y-1 text-sm flex-1">
          <a href="#" className="block px-3 py-2 bg-indigo-50 text-indigo-700 font-medium rounded-lg">All Logins</a>
        </nav>
        <button
          onClick={handleLock}
          className="mt-auto px-3 py-2 text-sm text-gray-600 hover:bg-gray-100 rounded-lg transition-colors text-left"
        >
          Lock Vault
        </button>
      </div>

      <div className="flex-1 p-8 overflow-auto">
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-2xl font-semibold text-gray-900">Vault</h1>
          <button
            onClick={() => setIsAddingItem(true)}
            className="bg-indigo-500 text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-indigo-600 transition-colors"
          >
            + Add Login
          </button>
        </div>

        {items.length === 0 ? (
          <div className="bg-white rounded-xl shadow-sm border border-gray-100 text-center p-12">
            <p className="text-gray-500 mb-4">Your vault is empty.</p>
            <button 
              onClick={() => setIsAddingItem(true)}
              className="bg-indigo-500 text-white px-4 py-2.5 rounded-lg text-sm font-medium hover:bg-indigo-600 transition-colors"
            >
              Add your first item
            </button>
          </div>
        ) : (
          <div className="grid gap-4">
            {items.map((item) => (
              <div key={item.id} className="bg-white p-4 rounded-xl shadow-sm border border-gray-100 flex items-center justify-between">
                <div>
                  <h3 className="font-medium text-gray-900">{item.name}</h3>
                  <p className="text-sm text-gray-500">{item.username}</p>
                </div>
                <div className="text-right">
                  <div className="font-mono text-sm bg-gray-100 px-2 py-1 rounded text-gray-600 select-all">
                    {item.password_str}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {isAddingItem && (
        <div className="absolute inset-0 bg-black/20 flex items-center justify-center p-4">
          <div className="bg-white rounded-xl shadow-lg border border-gray-100 p-6 w-full max-w-md">
            <h2 className="text-lg font-semibold mb-4">Add New Login</h2>
            <form onSubmit={handleAddItem} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Name</label>
                <input
                  required
                  type="text"
                  value={newItem.name}
                  onChange={(e) => setNewItem({ ...newItem, name: e.target.value })}
                  className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                  placeholder="e.g. Google, GitHub"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Username</label>
                <input
                  required
                  type="text"
                  value={newItem.username}
                  onChange={(e) => setNewItem({ ...newItem, username: e.target.value })}
                  className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                  placeholder="user@example.com"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Password</label>
                <input
                  required
                  type="password"
                  value={newItem.password_str}
                  onChange={(e) => setNewItem({ ...newItem, password_str: e.target.value })}
                  className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                  placeholder="••••••••"
                />
              </div>
              <div className="flex gap-3 mt-6">
                <button
                  type="button"
                  onClick={() => setIsAddingItem(false)}
                  className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="flex-1 bg-indigo-500 text-white px-4 py-2 rounded-lg text-sm font-medium hover:bg-indigo-600 transition-colors"
                >
                  Save Item
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
}
