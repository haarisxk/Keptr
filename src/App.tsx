import { useState } from 'react';

export default function App() {
  const [isLocked, setIsLocked] = useState(true);

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
                className="w-full rounded-lg border border-gray-300 px-3 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                placeholder="Enter master password"
              />
            </div>

            <button
              onClick={() => setIsLocked(false)}
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
            onClick={() => setIsLocked(true)}
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
