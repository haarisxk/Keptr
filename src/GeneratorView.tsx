import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Shield, RefreshCw, Copy, Check, Hash, Type, Mail, Link, Trash2 } from 'lucide-react';

interface GeneratorHistoryItem {
  id: string;
  value: string;
  type: 'password' | 'username';
  timestamp: number;
}

export default function GeneratorView() {
  const [activeTab, setActiveTab] = useState<'password' | 'username'>('password');
  
  // Password Options
  const [pwdType, setPwdType] = useState<'Chars' | 'Passphrase' | 'Pronounceable'>('Chars');
  const [pwdLength, setPwdLength] = useState(16);
  const [pwdUpper, setPwdUpper] = useState(true);
  const [pwdLower, setPwdLower] = useState(true);
  const [pwdNums, setPwdNums] = useState(true);
  const [pwdSyms, setPwdSyms] = useState(true);
  const [pwdExcludeAmbiguous, setPwdExcludeAmbiguous] = useState(false);
  const [pwdWordCount, setPwdWordCount] = useState(4);
  const [pwdSeparator, setPwdSeparator] = useState('-');

  // Username Options
  const [usrType, setUsrType] = useState<'RandomChars' | 'Words' | 'EmailAlias' | 'CatchAll'>('Words');
  const [usrLength, setUsrLength] = useState(12);
  const [usrWordCount, setUsrWordCount] = useState(2);
  const [usrSeparator, setUsrSeparator] = useState('_');
  const [usrBaseEmail, setUsrBaseEmail] = useState('');
  const [usrAliasPrefix, setUsrAliasPrefix] = useState('keptr');
  const [usrDomain, setUsrDomain] = useState('');

  const [generatedValue, setGeneratedValue] = useState('');
  const [entropyBits, setEntropyBits] = useState(0);
  const [copied, setCopied] = useState(false);
  const [history, setHistory] = useState<GeneratorHistoryItem[]>([]);

  useEffect(() => {
    generate();
  }, [
    activeTab, pwdType, pwdLength, pwdUpper, pwdLower, pwdNums, pwdSyms, pwdExcludeAmbiguous, pwdWordCount, pwdSeparator,
    usrType, usrLength, usrWordCount, usrSeparator, usrBaseEmail, usrAliasPrefix, usrDomain
  ]);

  const calculateEntropy = (val: string, type: 'password' | 'username') => {
    if (activeTab === 'password' && pwdType === 'Passphrase') {
      return Math.round(pwdWordCount * Math.log2(7776));
    }
    if (activeTab === 'username' && usrType === 'Words') {
      return Math.round(usrWordCount * Math.log2(7776));
    }
    
    // Fallback for char based
    let pool = 0;
    if (activeTab === 'password' && pwdType === 'Chars') {
      if (pwdLower) pool += pwdExcludeAmbiguous ? 25 : 26;
      if (pwdUpper) pool += pwdExcludeAmbiguous ? 24 : 26;
      if (pwdNums) pool += pwdExcludeAmbiguous ? 8 : 10;
      if (pwdSyms) pool += 26;
    } else {
      pool = 36; // a-z0-9
    }
    
    if (pool === 0) return 0;
    const length = activeTab === 'password' && pwdType === 'Chars' ? pwdLength : val.length;
    return Math.round(length * Math.log2(pool));
  };

  const generate = async () => {
    try {
      let result = '';
      if (activeTab === 'password') {
        result = await invoke<string>('generate_advanced_password', {
          options: {
            pwd_type: pwdType,
            length: pwdLength,
            uppercase: pwdUpper,
            lowercase: pwdLower,
            numbers: pwdNums,
            symbols: pwdSyms,
            exclude_ambiguous: pwdExcludeAmbiguous,
            word_count: pwdWordCount,
            separator: pwdSeparator,
          }
        });
      } else {
        if (usrType === 'EmailAlias' && !usrBaseEmail.includes('@')) {
           return; // Don't generate if invalid base
        }
        if (usrType === 'CatchAll' && !usrDomain) {
           return;
        }
        result = await invoke<string>('generate_username', {
          options: {
            usr_type: usrType,
            length: usrLength,
            word_count: usrWordCount,
            separator: usrSeparator,
            base_email: usrBaseEmail,
            alias_prefix: usrAliasPrefix,
            domain: usrDomain,
          }
        });
      }
      setGeneratedValue(result);
      setEntropyBits(calculateEntropy(result, activeTab));
      addToHistory(result, activeTab);
    } catch (e) {
      console.error("Generator Error:", e);
    }
  };

  const addToHistory = (value: string, type: 'password' | 'username') => {
    if (!value) return;
    setHistory(prev => {
      // Don't add if it's the same as the most recent one
      if (prev.length > 0 && prev[0].value === value) return prev;
      const newHist = [{ id: Math.random().toString(), value, type, timestamp: Date.now() }, ...prev];
      return newHist.slice(0, 10); // Keep last 10
    });
  };

  const handleCopy = (val: string) => {
    navigator.clipboard.writeText(val);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getEntropyLabel = (bits: number) => {
    if (bits < 40) return { label: 'Weak', color: 'bg-red-500' };
    if (bits < 60) return { label: 'Moderate', color: 'bg-yellow-500' };
    if (bits < 80) return { label: 'Strong', color: 'bg-emerald-500' };
    return { label: 'Overkill', color: 'bg-brand-500' };
  };

  const entropyStatus = getEntropyLabel(entropyBits);

  return (
    <div className="flex-1 overflow-y-auto p-8 animate-fade-in text-gray-200">
      <div className="max-w-3xl mx-auto space-y-8">
        
        {/* Header & Tabs */}
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-semibold text-white">Generator</h2>
          <div className="flex bg-dark-800 p-1 rounded-xl border border-dark-700">
            <button 
              onClick={() => setActiveTab('password')}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${activeTab === 'password' ? 'bg-dark-600 text-white shadow' : 'text-gray-400 hover:text-gray-200'}`}
            >
              Password
            </button>
            <button 
              onClick={() => setActiveTab('username')}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${activeTab === 'username' ? 'bg-dark-600 text-white shadow' : 'text-gray-400 hover:text-gray-200'}`}
            >
              Username
            </button>
          </div>
        </div>

        {/* The Output Area */}
        <div className="glass-panel rounded-2xl p-6 relative overflow-hidden">
          <div className="absolute top-0 left-0 w-full h-1 bg-dark-800">
            <div className={`h-full ${entropyStatus.color} transition-all duration-500`} style={{ width: `${Math.min(100, (entropyBits / 128) * 100)}%` }}></div>
          </div>
          <div className="flex flex-col md:flex-row gap-6 items-center">
            <div className="flex-1 w-full relative">
              <input 
                type="text" 
                readOnly 
                value={generatedValue}
                className="w-full bg-dark-950 border border-dark-600 rounded-xl pl-4 pr-16 py-4 font-mono text-xl text-white focus:outline-none"
              />
              <button 
                onClick={() => handleCopy(generatedValue)}
                className="absolute right-3 top-3.5 p-2 bg-dark-800 hover:bg-dark-700 text-gray-300 hover:text-white rounded-lg transition-colors border border-dark-600"
              >
                {copied ? <Check className="w-4 h-4 text-emerald-400" /> : <Copy className="w-4 h-4" />}
              </button>
            </div>
            <div className="flex flex-col items-center md:items-end flex-shrink-0">
              <span className={`text-lg font-semibold ${entropyStatus.color.replace('bg-', 'text-')}`}>{entropyBits} bits</span>
              <span className="text-sm text-gray-500 uppercase tracking-widest">{entropyStatus.label}</span>
            </div>
            <button 
              onClick={generate}
              className="bg-brand-500 hover:bg-brand-600 text-white p-4 rounded-xl transition-all shadow-lg shadow-brand-500/20"
            >
              <RefreshCw className="w-6 h-6" />
            </button>
          </div>
        </div>

        {/* Configuration Area */}
        <div className="glass-card rounded-2xl p-6 space-y-6">
          {activeTab === 'password' ? (
            <>
              {/* Password Types */}
              <div className="flex flex-wrap gap-3">
                {[
                  { id: 'Chars', label: 'Random Characters', icon: Hash },
                  { id: 'Passphrase', label: 'EFF Passphrase', icon: Type },
                  { id: 'Pronounceable', label: 'Pronounceable', icon: Mail }
                ].map(type => (
                  <button 
                    key={type.id}
                    onClick={() => setPwdType(type.id as any)}
                    className={`flex items-center gap-2 px-4 py-2.5 rounded-xl text-sm font-medium transition-all border ${pwdType === type.id ? 'bg-brand-500/10 border-brand-500/50 text-brand-400' : 'bg-dark-900 border-dark-700 text-gray-400 hover:text-gray-200'}`}
                  >
                    <type.icon className="w-4 h-4" /> {type.label}
                  </button>
                ))}
              </div>

              {pwdType === 'Chars' && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 pt-4">
                  <div>
                    <label className="flex justify-between text-sm font-medium text-gray-400 mb-4">
                      <span>Length</span>
                      <span className="text-white">{pwdLength}</span>
                    </label>
                    <input 
                      type="range" min="4" max="128" 
                      value={pwdLength} onChange={(e) => setPwdLength(Number(e.target.value))}
                      className="w-full accent-brand-500"
                    />
                  </div>
                  <div className="space-y-3">
                    <label className="flex items-center gap-3 text-sm text-gray-300 cursor-pointer">
                      <input type="checkbox" checked={pwdUpper} onChange={(e) => setPwdUpper(e.target.checked)} className="rounded border-dark-600 text-brand-500 bg-dark-900 cursor-pointer" /> A-Z (Uppercase)
                    </label>
                    <label className="flex items-center gap-3 text-sm text-gray-300 cursor-pointer">
                      <input type="checkbox" checked={pwdLower} onChange={(e) => setPwdLower(e.target.checked)} className="rounded border-dark-600 text-brand-500 bg-dark-900 cursor-pointer" /> a-z (Lowercase)
                    </label>
                    <label className="flex items-center gap-3 text-sm text-gray-300 cursor-pointer">
                      <input type="checkbox" checked={pwdNums} onChange={(e) => setPwdNums(e.target.checked)} className="rounded border-dark-600 text-brand-500 bg-dark-900 cursor-pointer" /> 0-9 (Numbers)
                    </label>
                    <label className="flex items-center gap-3 text-sm text-gray-300 cursor-pointer">
                      <input type="checkbox" checked={pwdSyms} onChange={(e) => setPwdSyms(e.target.checked)} className="rounded border-dark-600 text-brand-500 bg-dark-900 cursor-pointer" /> !@#$ (Symbols)
                    </label>
                    <label className="flex items-center gap-3 text-sm text-gray-400 pt-2 border-t border-dark-700 cursor-pointer">
                      <input type="checkbox" checked={pwdExcludeAmbiguous} onChange={(e) => setPwdExcludeAmbiguous(e.target.checked)} className="rounded border-dark-600 text-brand-500 bg-dark-900 cursor-pointer" /> Exclude ambiguous (l, 1, O, 0)
                    </label>
                  </div>
                </div>
              )}

              {pwdType === 'Passphrase' && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 pt-4">
                  <div>
                    <label className="flex justify-between text-sm font-medium text-gray-400 mb-4">
                      <span>Word Count</span>
                      <span className="text-white">{pwdWordCount}</span>
                    </label>
                    <input 
                      type="range" min="3" max="12" 
                      value={pwdWordCount} onChange={(e) => setPwdWordCount(Number(e.target.value))}
                      className="w-full accent-brand-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-400 mb-2">Word Separator</label>
                    <input 
                      type="text" value={pwdSeparator} onChange={(e) => setPwdSeparator(e.target.value)}
                      className="w-full bg-dark-900 border border-dark-700 rounded-xl px-4 py-2.5 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none"
                    />
                  </div>
                </div>
              )}

              {pwdType === 'Pronounceable' && (
                <div className="pt-4">
                  <label className="flex justify-between text-sm font-medium text-gray-400 mb-4">
                    <span>Length</span>
                    <span className="text-white">{pwdLength}</span>
                  </label>
                  <input 
                    type="range" min="4" max="64" 
                    value={pwdLength} onChange={(e) => setPwdLength(Number(e.target.value))}
                    className="w-full accent-brand-500 max-w-sm"
                  />
                </div>
              )}
            </>
          ) : (
            <>
              {/* Username Types */}
              <div className="flex flex-wrap gap-3">
                {[
                  { id: 'Words', label: 'Random Words', icon: Type },
                  { id: 'RandomChars', label: 'Alphanumeric', icon: Hash },
                  { id: 'EmailAlias', label: 'Email Alias (+)', icon: Mail },
                  { id: 'CatchAll', label: 'Catch-All Domain', icon: Link }
                ].map(type => (
                  <button 
                    key={type.id}
                    onClick={() => setUsrType(type.id as any)}
                    className={`flex items-center gap-2 px-4 py-2.5 rounded-xl text-sm font-medium transition-all border ${usrType === type.id ? 'bg-brand-500/10 border-brand-500/50 text-brand-400' : 'bg-dark-900 border-dark-700 text-gray-400 hover:text-gray-200'}`}
                  >
                    <type.icon className="w-4 h-4" /> {type.label}
                  </button>
                ))}
              </div>

              {usrType === 'Words' && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 pt-4">
                  <div>
                    <label className="flex justify-between text-sm font-medium text-gray-400 mb-4">
                      <span>Word Count</span>
                      <span className="text-white">{usrWordCount}</span>
                    </label>
                    <input 
                      type="range" min="2" max="5" 
                      value={usrWordCount} onChange={(e) => setUsrWordCount(Number(e.target.value))}
                      className="w-full accent-brand-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-400 mb-2">Word Separator</label>
                    <input 
                      type="text" value={usrSeparator} onChange={(e) => setUsrSeparator(e.target.value)}
                      className="w-full bg-dark-900 border border-dark-700 rounded-xl px-4 py-2.5 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none"
                    />
                  </div>
                </div>
              )}

              {usrType === 'EmailAlias' && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-8 pt-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-400 mb-2">Base Email</label>
                    <input 
                      type="email" placeholder="you@gmail.com" value={usrBaseEmail} onChange={(e) => setUsrBaseEmail(e.target.value)}
                      className="w-full bg-dark-900 border border-dark-700 rounded-xl px-4 py-2.5 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-400 mb-2">Alias Prefix (Optional)</label>
                    <input 
                      type="text" placeholder="keptr" value={usrAliasPrefix} onChange={(e) => setUsrAliasPrefix(e.target.value)}
                      className="w-full bg-dark-900 border border-dark-700 rounded-xl px-4 py-2.5 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none"
                    />
                  </div>
                </div>
              )}

              {usrType === 'CatchAll' && (
                <div className="pt-4">
                  <label className="block text-sm font-medium text-gray-400 mb-2">Your Custom Domain</label>
                  <input 
                    type="text" placeholder="@yourdomain.com" value={usrDomain} onChange={(e) => setUsrDomain(e.target.value)}
                    className="w-full max-w-sm bg-dark-900 border border-dark-700 rounded-xl px-4 py-2.5 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none"
                  />
                </div>
              )}
            </>
          )}
        </div>

        {/* Ephemeral History */}
        {history.length > 0 && (
          <div className="pt-6 border-t border-dark-800">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wider">Session History</h3>
              <button onClick={() => setHistory([])} className="text-xs text-gray-500 hover:text-gray-300 flex items-center gap-1">
                <Trash2 className="w-3 h-3" /> Clear
              </button>
            </div>
            <div className="space-y-2">
              {history.map(item => (
                <div key={item.id} className="flex justify-between items-center bg-dark-900/50 rounded-lg p-3 border border-dark-800 hover:border-dark-700 transition-colors">
                  <span className="font-mono text-sm text-gray-300 truncate max-w-[70%]">{item.value}</span>
                  <button onClick={() => handleCopy(item.value)} className="text-gray-500 hover:text-white p-1">
                    <Copy className="w-4 h-4" />
                  </button>
                </div>
              ))}
            </div>
          </div>
        )}

      </div>
    </div>
  );
}
