import React, { useState } from 'react';

export default function App() {
  const [isLocked, setIsLocked] = useState(true);

  if (isLocked) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-surface-secondary p-4">
        <div className="card w-full max-w-sm space-y-6">
          <div className="text-center">
            <h1 className="text-2xl font-semibold mb-1">Keptr</h1>
            <p className="text-sm text-gray-500">Your Digital Life, Kept Secure.</p>
          </div>
          
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium mb-1" htmlFor="master-password">
                Master Password
              </label>
              <input 
                id="master-password"
                type="password" 
                className="input-secure"
                placeholder="Enter master password"
              />
            </div>
            
            <button 
              onClick={() => setIsLocked(false)}
              className="btn-primary w-full"
            >
              Unlock Vault
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen flex bg-surface-secondary">
      {/* Sidebar */}
      <div className="w-64 bg-surface border-r border-gray-200 p-4">
        <h2 className="text-lg font-semibold mb-6">Keptr</h2>
        <nav className="space-y-2 text-sm">
          <a href="#" className="block px-3 py-2 bg-brand text-white rounded-md">All Items</a>
          <a href="#" className="block px-3 py-2 text-gray-600 hover:bg-gray-100 rounded-md">Logins</a>
          <a href="#" className="block px-3 py-2 text-gray-600 hover:bg-gray-100 rounded-md">Secure Notes</a>
          <a href="#" className="block px-3 py-2 text-gray-600 hover:bg-gray-100 rounded-md">Passkeys</a>
        </nav>
      </div>

      {/* Main Content */}
      <div className="flex-1 p-8">
        <div className="flex justify-between items-center mb-6">
          <h1 className="text-2xl font-semibold">Vault</h1>
          <button 
            onClick={() => setIsLocked(true)}
            className="px-4 py-2 text-sm text-gray-600 hover:bg-gray-200 rounded-md transition-colors"
          >
            Lock Vault
          </button>
        </div>
        
        <div className="card text-center p-12">
          <p className="text-gray-500 mb-4">Your vault is empty.</p>
          <button className="btn-primary">Add Item</button>
        </div>
      </div>
    </div>
  );
}
