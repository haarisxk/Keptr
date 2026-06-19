import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { 
  Key, Shield, Copy, Eye, EyeOff, Plus, Lock, 
  Search, Globe, ChevronRight, Check, AlertCircle,
  LayoutGrid, Settings, FileText, Fingerprint, Star
} from 'lucide-react';

interface LoginItemDto {
  id: string;
  name: string;
  url: string;
  username: string;
  password_str: string;
  notes: string;
}

// Reusable Abstract Pattern for the Auth Screens
const AbstractAuthBackground = () => (
  <div className="hidden lg:flex w-1/2 relative bg-dark-950 overflow-hidden items-center justify-center">
    {/* Animated glowing orbs */}
    <div className="absolute top-1/3 left-1/4 w-96 h-96 bg-brand-600/20 rounded-full blur-[120px] animate-pulse-slow"></div>
    <div className="absolute bottom-1/4 right-1/4 w-80 h-80 bg-indigo-500/10 rounded-full blur-[100px] animate-pulse-slow" style={{ animationDelay: '2s' }}></div>
    
    <div className="z-10 text-center max-w-md px-8 animate-fade-in-up">
      <div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-white/[0.03] border border-white/10 mb-8 backdrop-blur-md">
        <Shield className="w-8 h-8 text-brand-400" strokeWidth={1.5} />
      </div>
      <h2 className="text-3xl font-semibold text-white mb-4 tracking-tight">Zero-Knowledge Security</h2>
      <p className="text-gray-400 leading-relaxed">
        Your data is encrypted with military-grade XChaCha20-Poly1305 before it ever touches the database. 
        Only you hold the key.
      </p>
    </div>
    
    {/* Grid overlay */}
    <div className="absolute inset-0 bg-[url('data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAiIGhlaWdodD0iNDAiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PGcgc3Ryb2tlPSJyZ2JhKDI1NSwgMjU1LCAyNTUsIDAuMDMpIiBmaWxsPSJub25lIj48cGF0aCBkPSJNMCAwaDQwdjQwSDB6Ii8+PC9nPjwvc3ZnPg==')] opacity-50"></div>
  </div>
);

const PasswordItem = ({ item }: { item: LoginItemDto }) => {
  const [showPassword, setShowPassword] = useState(false);
  const [copied, setCopied] = useState(false);
  const [isFavorite, setIsFavorite] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(item.password_str);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="glass-card rounded-2xl p-5 flex items-center justify-between group animate-fade-in-up">
      <div className="flex items-center gap-4">
        <div className="w-12 h-12 rounded-xl bg-dark-700/50 border border-dark-600 flex items-center justify-center flex-shrink-0">
          <Globe className="w-5 h-5 text-gray-400" />
        </div>
        <div>
          <h3 className="text-base font-medium text-white mb-0.5">{item.name}</h3>
          <p className="text-sm text-gray-500">{item.username}</p>
        </div>
      </div>
      
      <div className="flex items-center gap-6">
        <div className="flex items-center gap-2">
          <div className="font-mono text-sm text-gray-400 bg-dark-900/50 px-3 py-1.5 rounded-lg border border-dark-600/50 w-32 text-center">
            {showPassword ? item.password_str : '••••••••••••'}
          </div>
          <button 
            onClick={() => setShowPassword(!showPassword)}
            className="p-2 text-gray-500 hover:text-gray-300 hover:bg-dark-700 rounded-lg transition-colors"
            title={showPassword ? "Hide password" : "Show password"}
          >
            {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
          </button>
          <button 
            onClick={handleCopy}
            className="p-2 text-gray-500 hover:text-white hover:bg-dark-700 rounded-lg transition-colors relative"
            title="Copy password"
          >
            {copied ? <Check className="w-4 h-4 text-emerald-400" /> : <Copy className="w-4 h-4" />}
          </button>
          <button 
            onClick={() => setIsFavorite(!isFavorite)}
            className="p-2 text-gray-500 hover:text-white hover:bg-dark-700 rounded-lg transition-colors relative"
            title={isFavorite ? "Remove from favorites" : "Add to favorites"}
          >
            <Star className={`w-4 h-4 ${isFavorite ? 'text-yellow-400 fill-yellow-400' : ''}`} />
          </button>
        </div>
        <button className="text-gray-600 hover:text-white transition-colors">
          <ChevronRight className="w-5 h-5" />
        </button>
      </div>
    </div>
  );
};

export default function App() {
  const [isInitialized, setIsInitialized] = useState<boolean | null>(null);
  const [isLocked, setIsLocked] = useState(true);
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');

  // Vault state
  const [items, setItems] = useState<LoginItemDto[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [isAddingItem, setIsAddingItem] = useState(false);
  const [newItem, setNewItem] = useState({
    name: '', url: '', username: '', password_str: '', notes: '',
  });

  useEffect(() => {
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

  const handleInitialize = async (e: React.FormEvent) => {
    e.preventDefault();
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

  const handleUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
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

  const filteredItems = items.filter(item => 
    item.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
    item.username.toLowerCase().includes(searchQuery.toLowerCase())
  );

  if (isInitialized === null) {
    return <div className="min-h-screen bg-dark-900 flex items-center justify-center text-gray-500">Initializing...</div>;
  }

  // --- SETUP SCREEN ---
  if (!isInitialized) {
    return (
      <div className="min-h-screen flex bg-dark-900 animate-fade-in">
        <AbstractAuthBackground />
        <div className="w-full lg:w-1/2 flex items-center justify-center p-8 lg:p-24 relative">
          <div className="w-full max-w-md animate-slide-in">
            <div className="mb-12">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 rounded-xl bg-brand-500 flex items-center justify-center shadow-lg shadow-brand-500/30">
                  <Key className="w-5 h-5 text-white" />
                </div>
                <h1 className="text-2xl font-semibold text-white tracking-tight">Keptr</h1>
              </div>
              <h2 className="text-3xl font-semibold text-white mb-2">Create your vault</h2>
              <p className="text-gray-400">Set a strong master password. This is the only key that can decrypt your data.</p>
            </div>

            <form onSubmit={handleInitialize} className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">Master Password</label>
                <div className="relative">
                  <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="input-field pl-12 pr-4"
                    placeholder="Enter a strong password"
                    autoFocus
                  />
                  <Lock className="w-5 h-5 text-gray-500 absolute left-4 top-3.5" />
                </div>
              </div>
              
              {error && (
                <div className="flex items-center gap-2 text-red-400 text-sm bg-red-400/10 p-3 rounded-lg border border-red-400/20">
                  <AlertCircle className="w-4 h-4" />
                  {error}
                </div>
              )}

              <button type="submit" className="w-full bg-brand-500 hover:bg-brand-600 text-white font-medium py-3.5 px-4 rounded-xl transition-all duration-200 brand-glow flex justify-center items-center gap-2">
                Initialize Vault <ChevronRight className="w-4 h-4" />
              </button>
            </form>
          </div>
        </div>
      </div>
    );
  }

  // --- UNLOCK SCREEN ---
  if (isLocked) {
    return (
      <div className="min-h-screen flex bg-dark-900 animate-fade-in">
        <AbstractAuthBackground />
        <div className="w-full lg:w-1/2 flex items-center justify-center p-8 lg:p-24 relative">
          <div className="w-full max-w-md animate-slide-in">
            <div className="mb-12">
              <div className="flex items-center gap-3 mb-6">
                <div className="w-10 h-10 rounded-xl bg-brand-500 flex items-center justify-center shadow-lg shadow-brand-500/30">
                  <Fingerprint className="w-5 h-5 text-white" />
                </div>
                <h1 className="text-2xl font-semibold text-white tracking-tight">Keptr</h1>
              </div>
              <h2 className="text-3xl font-semibold text-white mb-2">Welcome back</h2>
              <p className="text-gray-400">Enter your master password to unlock your secure vault.</p>
            </div>

            <form onSubmit={handleUnlock} className="space-y-6">
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">Master Password</label>
                <div className="relative">
                  <input
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="input-field pl-12 pr-4"
                    placeholder="••••••••••••"
                    autoFocus
                  />
                  <Lock className="w-5 h-5 text-gray-500 absolute left-4 top-3.5" />
                </div>
              </div>
              
              {error && (
                <div className="flex items-center gap-2 text-red-400 text-sm bg-red-400/10 p-3 rounded-lg border border-red-400/20">
                  <AlertCircle className="w-4 h-4" />
                  {error}
                </div>
              )}

              <button type="submit" className="w-full bg-brand-500 hover:bg-brand-600 text-white font-medium py-3.5 px-4 rounded-xl transition-all duration-200 brand-glow flex justify-center items-center gap-2">
                Unlock Vault <ChevronRight className="w-4 h-4" />
              </button>
            </form>
          </div>
        </div>
      </div>
    );
  }

  // --- DASHBOARD ---
  return (
    <div className="min-h-screen bg-dark-950 flex text-gray-200 overflow-hidden animate-fade-in">
      {/* Sidebar */}
      <aside className="w-72 bg-dark-900 border-r border-dark-800 flex flex-col">
        <div className="h-20 flex items-center px-6 border-b border-dark-800">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-brand-500 flex items-center justify-center shadow-lg shadow-brand-500/20">
              <Key className="w-4 h-4 text-white" />
            </div>
            <span className="text-lg font-semibold text-white tracking-tight">Keptr</span>
          </div>
        </div>

        <div className="p-4 flex-1 overflow-y-auto hide-scrollbar">
          <nav className="space-y-1">
            <p className="px-3 text-xs font-semibold text-gray-500 uppercase tracking-wider mb-2 mt-4">Vault</p>
            <a href="#" className="flex items-center gap-3 px-3 py-2.5 bg-brand-500/10 text-brand-400 rounded-xl font-medium transition-colors">
              <LayoutGrid className="w-5 h-5" /> All Items
            </a>
            <a href="#" className="flex items-center gap-3 px-3 py-2.5 text-gray-400 hover:text-gray-200 hover:bg-dark-800 rounded-xl font-medium transition-colors">
              <Globe className="w-5 h-5" /> Logins
            </a>
            <a href="#" className="flex items-center gap-3 px-3 py-2.5 text-gray-400 hover:text-gray-200 hover:bg-dark-800 rounded-xl font-medium transition-colors">
              <FileText className="w-5 h-5" /> Secure Notes
            </a>
          </nav>
        </div>

        <div className="p-4 border-t border-dark-800">
          <button className="flex items-center gap-3 px-3 py-2.5 w-full text-gray-400 hover:text-gray-200 hover:bg-dark-800 rounded-xl font-medium transition-colors mb-2">
            <Settings className="w-5 h-5" /> Settings
          </button>
          <button 
            onClick={handleLock}
            className="flex items-center gap-3 px-3 py-2.5 w-full text-red-400 hover:bg-red-400/10 rounded-xl font-medium transition-colors"
          >
            <Lock className="w-5 h-5" /> Lock Vault
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col min-w-0">
        {/* Topbar */}
        <header className="h-20 flex items-center justify-between px-8 bg-dark-900/50 backdrop-blur-md border-b border-dark-800 sticky top-0 z-10 gap-8">
          <div className="relative flex-1">
            <Search className="w-5 h-5 text-gray-500 absolute left-4 top-2.5" />
            <input 
              type="text" 
              placeholder="Search your vault..." 
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-dark-800 border border-dark-700 rounded-xl pl-12 pr-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500/50 transition-all placeholder:text-gray-500"
            />
          </div>
          <button 
            onClick={() => setIsAddingItem(true)}
            className="bg-brand-500 hover:bg-brand-600 text-white px-5 py-2.5 rounded-xl text-sm font-medium transition-all shadow-lg shadow-brand-500/20 flex items-center gap-2 flex-shrink-0"
          >
            <Plus className="w-4 h-4" /> New Item
          </button>
        </header>

        {/* List Content */}
        <div className="flex-1 overflow-y-auto p-8">
          <div className="w-full">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-semibold text-white">All Items</h2>
              <span className="text-sm text-gray-500">{filteredItems.length} items securely stored</span>
            </div>

            {filteredItems.length === 0 ? (
              <div className="glass-card rounded-2xl p-16 flex flex-col items-center justify-center text-center border-dashed border-dark-600">
                <div className="w-16 h-16 rounded-2xl bg-dark-800 flex items-center justify-center mb-6">
                  <Shield className="w-8 h-8 text-gray-500" />
                </div>
                <h3 className="text-lg font-medium text-white mb-2">Nothing here yet</h3>
                <p className="text-gray-400 mb-6 max-w-sm">
                  Add your first login, secure note, or identity to start building your encrypted vault.
                </p>
                <button 
                  onClick={() => setIsAddingItem(true)}
                  className="bg-dark-800 hover:bg-dark-700 text-white px-5 py-2.5 rounded-xl text-sm font-medium transition-colors border border-dark-600 flex items-center gap-2"
                >
                  <Plus className="w-4 h-4" /> Add Item
                </button>
              </div>
            ) : (
              <div className="grid gap-3">
                {filteredItems.map(item => (
                  <PasswordItem key={item.id} item={item} />
                ))}
              </div>
            )}
          </div>
        </div>
      </main>

      {/* Add Item Modal */}
      {isAddingItem && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-dark-950/80 backdrop-blur-sm animate-fade-in">
          <div className="glass-panel w-full max-w-md rounded-2xl overflow-hidden animate-fade-in-up">
            <div className="px-6 py-4 border-b border-white/5 flex justify-between items-center bg-white/[0.02]">
              <h2 className="text-lg font-semibold text-white">Add New Login</h2>
              <button onClick={() => setIsAddingItem(false)} className="text-gray-400 hover:text-white transition-colors">
                ✕
              </button>
            </div>
            
            <form onSubmit={handleAddItem} className="p-6 space-y-5">
              <div>
                <label className="block text-sm font-medium text-gray-400 mb-1.5">Name</label>
                <input
                  required
                  type="text"
                  value={newItem.name}
                  onChange={(e) => setNewItem({ ...newItem, name: e.target.value })}
                  className="input-field px-4"
                  placeholder="e.g. Google, GitHub"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-400 mb-1.5">Username / Email</label>
                <input
                  required
                  type="text"
                  value={newItem.username}
                  onChange={(e) => setNewItem({ ...newItem, username: e.target.value })}
                  className="input-field px-4"
                  placeholder="user@example.com"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-400 mb-1.5">Password</label>
                <input
                  required
                  type="password"
                  value={newItem.password_str}
                  onChange={(e) => setNewItem({ ...newItem, password_str: e.target.value })}
                  className="input-field px-4"
                  placeholder="••••••••"
                />
              </div>
              
              <div className="pt-4 flex gap-3">
                <button
                  type="button"
                  onClick={() => setIsAddingItem(false)}
                  className="flex-1 py-3 text-sm font-medium text-gray-300 hover:text-white bg-dark-800 hover:bg-dark-700 rounded-xl transition-colors border border-white/5"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  className="flex-1 bg-brand-500 hover:bg-brand-600 text-white py-3 rounded-xl text-sm font-medium transition-all shadow-lg shadow-brand-500/20"
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
