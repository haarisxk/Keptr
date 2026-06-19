import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { RefreshCw, Copy, Check, Hash, Type, Mail, Link, Trash2, ShieldCheck } from 'lucide-react';

interface GeneratorHistoryItem {
  id: string;
  value: string;
  type: 'password' | 'username';
  timestamp: number;
}

const CustomCheckbox = ({ checked, onChange, label }: { checked: boolean, onChange: (c: boolean) => void, label: React.ReactNode }) => (
  <label className="flex items-center gap-3 text-sm text-gray-300 cursor-pointer group select-none">
    <div className={`w-5 h-5 rounded-md border flex items-center justify-center transition-all duration-200 ${checked ? 'bg-brand-500 border-brand-500 shadow-sm shadow-brand-500/30' : 'bg-dark-950 border-dark-600 group-hover:border-dark-500'}`}>
      {checked && <Check className="w-3.5 h-3.5 text-white" strokeWidth={3} />}
    </div>
    {label}
  </label>
);

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
    let bits = 0;
    if (activeTab === 'password' && pwdType === 'Passphrase') {
      bits = Math.round(pwdWordCount * Math.log2(7776));
    } else if (activeTab === 'username' && usrType === 'Words') {
      bits = Math.round(usrWordCount * Math.log2(7776));
    } else {
      let pool = 0;
      if (activeTab === 'password' && pwdType === 'Chars') {
        if (pwdLower) pool += pwdExcludeAmbiguous ? 25 : 26;
        if (pwdUpper) pool += pwdExcludeAmbiguous ? 24 : 26;
        if (pwdNums) pool += pwdExcludeAmbiguous ? 8 : 10;
        if (pwdSyms) pool += 26;
      } else {
        pool = 36; // a-z0-9
      }
      const length = activeTab === 'password' && pwdType === 'Chars' ? pwdLength : val.length;
      bits = pool > 0 ? Math.round(length * Math.log2(pool)) : 0;
    }
    return Math.min(bits, 256); // Cap at 256 bits
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
           return;
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
    } catch (e) {
      console.error("Generator Error:", e);
    }
  };

  const addToHistory = (value: string, type: 'password' | 'username') => {
    if (!value) return;
    setHistory(prev => {
      if (prev.length > 0 && prev[0].value === value) return prev;
      const newHist = [{ id: Math.random().toString(), value, type, timestamp: Date.now() }, ...prev];
      return newHist.slice(0, 20);
    });
  };

  const handleCopy = (val: string) => {
    navigator.clipboard.writeText(val);
    setCopied(true);
    addToHistory(val, activeTab);
    setTimeout(() => setCopied(false), 2000);
  };

  const getEntropyLabel = (bits: number) => {
    if (bits < 35) return { label: 'Weak', color: 'bg-red-500' };
    if (bits < 46) return { label: 'Good', color: 'bg-yellow-500' };
    if (bits < 60) return { label: 'Strong', color: 'bg-emerald-500' };
    if (bits < 128) return { label: 'Very Strong', color: 'bg-emerald-400' };
    return { label: 'Overkill', color: 'bg-brand-500' };
  };

  const entropyStatus = getEntropyLabel(entropyBits);

  return (
    <div className="flex h-full w-full animate-fade-in text-gray-200 overflow-hidden bg-dark-950">
      
      {/* Main Generator Content */}
      <div className="flex-1 overflow-y-auto p-10 hide-scrollbar">
        <div className="max-w-4xl mx-auto space-y-10">
          
          {/* Header & Tabs */}
          <div className="flex items-center justify-between">
            <h2 className="text-3xl font-semibold text-white tracking-tight">Generator</h2>
            <div className="flex bg-dark-900 p-1.5 rounded-xl border border-dark-800 shadow-inner">
              <button 
                onClick={() => setActiveTab('password')}
                className={`px-5 py-2.5 rounded-lg text-sm font-medium transition-all ${activeTab === 'password' ? 'bg-dark-700 text-white shadow-md' : 'text-gray-400 hover:text-gray-200'}`}
              >
                Password
              </button>
              <button 
                onClick={() => setActiveTab('username')}
                className={`px-5 py-2.5 rounded-lg text-sm font-medium transition-all ${activeTab === 'username' ? 'bg-dark-700 text-white shadow-md' : 'text-gray-400 hover:text-gray-200'}`}
              >
                Username
              </button>
            </div>
          </div>

          {/* The Output Area */}
          <div className="glass-panel rounded-3xl p-8 relative overflow-hidden shadow-2xl">
            <div className="absolute top-0 left-0 w-full h-1.5 bg-dark-900">
              <div className={`h-full ${entropyStatus.color} transition-all duration-500 ease-out`} style={{ width: `${Math.min(100, (entropyBits / 256) * 100)}%` }}></div>
            </div>
            
            <div className="flex flex-col lg:flex-row gap-8 items-center">
              <div className="flex-1 w-full relative group">
                <input 
                  type="text" 
                  readOnly 
                  value={generatedValue}
                  className="w-full bg-dark-950/50 border border-dark-700 rounded-2xl pl-6 pr-20 py-5 font-mono text-2xl text-white focus:outline-none focus:ring-2 focus:ring-brand-500/30 transition-all shadow-inner"
                />
                <button 
                  onClick={() => handleCopy(generatedValue)}
                  className="absolute right-4 top-4 p-3 bg-dark-800 hover:bg-dark-700 text-gray-300 hover:text-white rounded-xl transition-all border border-dark-600 shadow-md group-hover:scale-105"
                  title="Copy and save to history"
                >
                  {copied ? <Check className="w-5 h-5 text-emerald-400" /> : <Copy className="w-5 h-5" />}
                </button>
              </div>
              
              <div className="flex lg:flex-col items-center lg:items-end justify-between w-full lg:w-auto gap-4">
                <div className="flex flex-col items-start lg:items-end">
                  <span className={`text-2xl font-bold ${entropyStatus.color.replace('bg-', 'text-')}`}>{entropyBits} bits</span>
                  <span className="text-sm font-medium text-gray-500 uppercase tracking-widest">{entropyStatus.label}</span>
                </div>
                <button 
                  onClick={generate}
                  className="bg-brand-500 hover:bg-brand-600 text-white p-4 rounded-xl transition-all shadow-lg shadow-brand-500/20 hover:shadow-brand-500/40 hover:-translate-y-0.5 active:translate-y-0"
                  title="Regenerate"
                >
                  <RefreshCw className="w-6 h-6" />
                </button>
              </div>
            </div>
          </div>

          {/* Configuration Area */}
          <div className="glass-card rounded-3xl p-8">
            {activeTab === 'password' ? (
              <div className="space-y-8">
                {/* Password Types */}
                <div className="flex flex-wrap gap-4">
                  {[
                    { id: 'Chars', label: 'Random Characters', icon: Hash },
                    { id: 'Passphrase', label: 'EFF Passphrase', icon: Type },
                    { id: 'Pronounceable', label: 'Pronounceable', icon: Mail }
                  ].map(type => (
                    <button 
                      key={type.id}
                      onClick={() => setPwdType(type.id as any)}
                      className={`flex items-center gap-2.5 px-5 py-3 rounded-xl text-sm font-medium transition-all border ${pwdType === type.id ? 'bg-brand-500/10 border-brand-500/50 text-brand-400 shadow-inner' : 'bg-dark-900/50 border-dark-700 text-gray-400 hover:text-gray-200 hover:bg-dark-800'}`}
                    >
                      <type.icon className="w-4 h-4" /> {type.label}
                    </button>
                  ))}
                </div>

                <div className="h-px bg-dark-800 w-full"></div>

                {pwdType === 'Chars' && (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
                    <div className="space-y-6">
                      <label className="flex justify-between text-sm font-medium text-gray-400">
                        <span>Password Length</span>
                        <span className="text-white font-mono bg-dark-800 px-3 py-1 rounded-lg">{pwdLength}</span>
                      </label>
                      <input 
                        type="range" min="4" max="128" 
                        value={pwdLength} onChange={(e) => setPwdLength(Number(e.target.value))}
                        className="w-full h-2 bg-dark-800 rounded-lg appearance-none cursor-pointer accent-brand-500"
                      />
                    </div>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-y-5 gap-x-6">
                      <CustomCheckbox checked={pwdUpper} onChange={setPwdUpper} label={<>A-Z <span className="text-gray-500">(Uppercase)</span></>} />
                      <CustomCheckbox checked={pwdLower} onChange={setPwdLower} label={<>a-z <span className="text-gray-500">(Lowercase)</span></>} />
                      <CustomCheckbox checked={pwdNums} onChange={setPwdNums} label={<>0-9 <span className="text-gray-500">(Numbers)</span></>} />
                      <CustomCheckbox checked={pwdSyms} onChange={setPwdSyms} label={<>!@#$ <span className="text-gray-500">(Symbols)</span></>} />
                      <div className="col-span-1 sm:col-span-2 pt-2 mt-2 border-t border-dark-800/50">
                        <CustomCheckbox checked={pwdExcludeAmbiguous} onChange={setPwdExcludeAmbiguous} label={<>Exclude ambiguous <span className="text-gray-500 font-mono">(l, 1, O, 0)</span></>} />
                      </div>
                    </div>
                  </div>
                )}

                {pwdType === 'Passphrase' && (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
                    <div className="space-y-6">
                      <label className="flex justify-between text-sm font-medium text-gray-400">
                        <span>Word Count</span>
                        <span className="text-white font-mono bg-dark-800 px-3 py-1 rounded-lg">{pwdWordCount}</span>
                      </label>
                      <input 
                        type="range" min="3" max="12" 
                        value={pwdWordCount} onChange={(e) => setPwdWordCount(Number(e.target.value))}
                        className="w-full h-2 bg-dark-800 rounded-lg appearance-none cursor-pointer accent-brand-500"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-400 mb-3">Word Separator</label>
                      <input 
                        type="text" value={pwdSeparator} onChange={(e) => setPwdSeparator(e.target.value)}
                        className="w-full max-w-[200px] bg-dark-900 border border-dark-700 rounded-xl px-4 py-3 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none"
                        placeholder="e.g. -"
                      />
                    </div>
                  </div>
                )}

                {pwdType === 'Pronounceable' && (
                  <div className="space-y-6 max-w-md">
                    <label className="flex justify-between text-sm font-medium text-gray-400">
                      <span>Length</span>
                      <span className="text-white font-mono bg-dark-800 px-3 py-1 rounded-lg">{pwdLength}</span>
                    </label>
                    <input 
                      type="range" min="4" max="64" 
                      value={pwdLength} onChange={(e) => setPwdLength(Number(e.target.value))}
                      className="w-full h-2 bg-dark-800 rounded-lg appearance-none cursor-pointer accent-brand-500"
                    />
                  </div>
                )}
              </div>
            ) : (
              <div className="space-y-8">
                {/* Username Types */}
                <div className="flex flex-wrap gap-4">
                  {[
                    { id: 'Words', label: 'Random Words', icon: Type },
                    { id: 'RandomChars', label: 'Alphanumeric', icon: Hash },
                    { id: 'EmailAlias', label: 'Email Alias (+)', icon: Mail },
                    { id: 'CatchAll', label: 'Catch-All Domain', icon: Link }
                  ].map(type => (
                    <button 
                      key={type.id}
                      onClick={() => setUsrType(type.id as any)}
                      className={`flex items-center gap-2.5 px-5 py-3 rounded-xl text-sm font-medium transition-all border ${usrType === type.id ? 'bg-brand-500/10 border-brand-500/50 text-brand-400 shadow-inner' : 'bg-dark-900/50 border-dark-700 text-gray-400 hover:text-gray-200 hover:bg-dark-800'}`}
                    >
                      <type.icon className="w-4 h-4" /> {type.label}
                    </button>
                  ))}
                </div>

                <div className="h-px bg-dark-800 w-full"></div>

                {usrType === 'Words' && (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
                    <div className="space-y-6">
                      <label className="flex justify-between text-sm font-medium text-gray-400">
                        <span>Word Count</span>
                        <span className="text-white font-mono bg-dark-800 px-3 py-1 rounded-lg">{usrWordCount}</span>
                      </label>
                      <input 
                        type="range" min="2" max="5" 
                        value={usrWordCount} onChange={(e) => setUsrWordCount(Number(e.target.value))}
                        className="w-full h-2 bg-dark-800 rounded-lg appearance-none cursor-pointer accent-brand-500"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-400 mb-3">Word Separator</label>
                      <input 
                        type="text" value={usrSeparator} onChange={(e) => setUsrSeparator(e.target.value)}
                        className="w-full max-w-[200px] bg-dark-900 border border-dark-700 rounded-xl px-4 py-3 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none"
                      />
                    </div>
                  </div>
                )}

                {usrType === 'RandomChars' && (
                  <div className="space-y-6 max-w-md">
                    <label className="flex justify-between text-sm font-medium text-gray-400">
                      <span>Length</span>
                      <span className="text-white font-mono bg-dark-800 px-3 py-1 rounded-lg">{usrLength}</span>
                    </label>
                    <input 
                      type="range" min="4" max="32" 
                      value={usrLength} onChange={(e) => setUsrLength(Number(e.target.value))}
                      className="w-full h-2 bg-dark-800 rounded-lg appearance-none cursor-pointer accent-brand-500"
                    />
                  </div>
                )}

                {usrType === 'EmailAlias' && (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    <div>
                      <label className="block text-sm font-medium text-gray-400 mb-3">Base Email</label>
                      <input 
                        type="email" placeholder="you@gmail.com" value={usrBaseEmail} onChange={(e) => setUsrBaseEmail(e.target.value)}
                        className="w-full bg-dark-900 border border-dark-700 rounded-xl px-4 py-3 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none transition-all"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-400 mb-3">Alias Prefix (Optional)</label>
                      <input 
                        type="text" placeholder="keptr" value={usrAliasPrefix} onChange={(e) => setUsrAliasPrefix(e.target.value)}
                        className="w-full bg-dark-900 border border-dark-700 rounded-xl px-4 py-3 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none transition-all"
                      />
                    </div>
                  </div>
                )}

                {usrType === 'CatchAll' && (
                  <div>
                    <label className="block text-sm font-medium text-gray-400 mb-3">Your Custom Domain</label>
                    <input 
                      type="text" placeholder="@yourdomain.com" value={usrDomain} onChange={(e) => setUsrDomain(e.target.value)}
                      className="w-full max-w-md bg-dark-900 border border-dark-700 rounded-xl px-4 py-3 text-white focus:ring-2 focus:ring-brand-500/50 focus:outline-none transition-all"
                    />
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Right Sidebar: Session History */}
      <aside className="w-80 bg-dark-900 border-l border-dark-800 flex flex-col flex-shrink-0 z-20">
        <div className="h-20 flex items-center px-6 border-b border-dark-800 bg-dark-900/90 backdrop-blur shrink-0">
          <div className="flex items-center gap-2">
            <ShieldCheck className="w-5 h-5 text-brand-400" />
            <h3 className="text-sm font-semibold text-gray-300 uppercase tracking-wider">Copied History</h3>
          </div>
          <button 
            onClick={() => setHistory([])} 
            className="ml-auto text-xs text-gray-500 hover:text-gray-300 flex items-center gap-1 bg-dark-800 px-2 py-1 rounded-md transition-colors"
          >
            <Trash2 className="w-3 h-3" /> Clear
          </button>
        </div>
        
        <div className="p-4 overflow-y-auto hide-scrollbar flex-1 space-y-3">
          {history.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-40 text-center px-4 border border-dashed border-dark-700 rounded-2xl bg-dark-900/50">
              <Copy className="w-8 h-8 text-dark-600 mb-3" />
              <p className="text-sm text-gray-500 leading-relaxed">
                Items you copy will be saved here until you lock the vault.
              </p>
            </div>
          ) : (
            history.map(item => (
              <div 
                key={item.id} 
                onClick={() => {
                  navigator.clipboard.writeText(item.value);
                }}
                className="group flex flex-col bg-dark-950/50 rounded-xl p-3 border border-dark-700/50 hover:border-brand-500/30 hover:bg-dark-800 transition-all cursor-pointer shadow-sm relative overflow-hidden"
              >
                <div className="absolute inset-y-0 left-0 w-1 bg-brand-500 opacity-0 group-hover:opacity-100 transition-opacity"></div>
                <div className="flex justify-between items-center w-full">
                  <span className="font-mono text-sm text-gray-200 truncate w-[90%]">{item.value}</span>
                  <Copy className="w-4 h-4 text-gray-600 group-hover:text-brand-400 transition-colors flex-shrink-0" />
                </div>
                <span className="text-[10px] text-gray-600 uppercase tracking-wider mt-2 font-medium">
                  {item.type} • {new Date(item.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                </span>
              </div>
            ))
          )}
        </div>
      </aside>

    </div>
  );
}
