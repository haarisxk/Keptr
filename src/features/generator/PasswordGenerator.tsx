import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Copy, RefreshCw, Clock, ShieldCheck, Check } from "lucide-react";
import {
    generateRandomPassword,
    generatePassphrase,
    generatePronounceable,
    calculateEntropy,
    GeneratorConfig
} from "./generator";
import { cn } from "@/lib/utils";
import { useSecureClipboard } from "@/features/security/useSecureClipboard";

export function PasswordGenerator() {
    const [mode, setMode] = useState<'random' | 'passphrase' | 'pronounceable'>('random');
    const [password, setPassword] = useState("");
    const [entropy, setEntropy] = useState(0);
    const [history, setHistory] = useState<string[]>([]);

    // Hook handles copy state and clearing
    const { copySecurely, isCopied } = useSecureClipboard();

    const [config, setConfig] = useState<GeneratorConfig>({
        length: 16,
        useUppercase: true,
        useLowercase: true,
        useNumbers: true,
        useSymbols: true,
        excludeAmbiguous: false,
        wordCount: 5,
        separator: "-",
        syllableCount: 5
    });

    useEffect(() => {
        generate();
    }, [config, mode]);

    const generate = () => {
        let newPassword = "";
        switch (mode) {
            case 'random':
                newPassword = generateRandomPassword(config);
                break;
            case 'passphrase':
                newPassword = generatePassphrase(config);
                break;
            case 'pronounceable':
                newPassword = generatePronounceable(config);
                break;
        }
        setPassword(newPassword);
        setEntropy(calculateEntropy(newPassword));
    };

    const handleCopy = (text: string) => {
        copySecurely(text);
        if (text && !history.includes(text)) {
            setHistory(prev => [text, ...prev].slice(0, 10));
        }
    };

    // Helper to update config
    const updateConfig = (key: keyof GeneratorConfig, value: any) => {
        setConfig(prev => ({ ...prev, [key]: value }));
    };

    const getStrengthColor = (ent: number) => {
        if (ent < 40) return "bg-red-500";
        if (ent < 60) return "bg-orange-500";
        if (ent < 80) return "bg-yellow-500";
        return "bg-green-500";
    };

    const getStrengthLabel = (ent: number) => {
        if (ent < 40) return "Weak";
        if (ent < 60) return "Fair";
        if (ent < 80) return "Good";
        return "Strong";
    };

    return (
        <div className="flex flex-col h-full bg-background/50 backdrop-blur-sm">
            {/* Header */}
            <div className="px-6 py-6 border-b flex items-center justify-between">
                <div>
                    <h2 className="text-2xl font-bold tracking-tight">Password Generator</h2>
                    <p className="text-muted-foreground">Create secure, strong passwords instantly.</p>
                </div>
                <div className="flex items-center gap-2 text-sm text-muted-foreground bg-muted px-3 py-1 rounded-full">
                    <ShieldCheck className="h-4 w-4" />
                    <span>Secure Generator</span>
                </div>
            </div>

            <div className="flex-1 flex overflow-hidden">
                {/* Main Content */}
                <div className="flex-1 flex flex-col p-8 overflow-y-auto max-w-4xl mx-auto w-full gap-8">

                    {/* Display Area */}
                    <div className="bg-card border rounded-xl p-6 shadow-sm space-y-4">
                        <div className="relative">
                            <div className="text-3xl font-mono break-all pr-12 font-medium tracking-tight text-center py-8">
                                {password}
                            </div>
                            <Button
                                variant="ghost"
                                size="icon"
                                className={cn("absolute top-0 right-0 h-8 w-8 text-muted-foreground hover:text-primary", isCopied && "text-green-500")}
                                onClick={() => handleCopy(password)}
                                title="Copy to Clipboard"
                            >
                                {isCopied ? <Check className="h-4 w-4" /> : <Copy className="h-4 w-4" />}
                            </Button>
                        </div>

                        {/* Strength Meter */}
                        <div className="space-y-2">
                            <div className="flex justify-between text-xs font-medium text-muted-foreground uppercase tracking-wider">
                                <span>Strength: {getStrengthLabel(entropy)}</span>
                                <span>{entropy} bits</span>
                            </div>
                            <div className="h-2 w-full bg-secondary rounded-full overflow-hidden">
                                <div
                                    className={cn("h-full transition-all duration-300", getStrengthColor(entropy))}
                                    style={{ width: `${Math.min(100, (entropy / 100) * 100)}%` }}
                                />
                            </div>
                        </div>

                        <div className="flex justify-center pt-2">
                            <Button onClick={generate} className="gap-2 px-8">
                                <RefreshCw className="h-4 w-4" /> Regenerate
                            </Button>
                        </div>
                    </div>

                    {/* Controls */}
                    <div className="bg-card border rounded-xl p-1 shadow-sm">
                        <Tabs value={mode} onValueChange={(v: any) => setMode(v)} className="w-full">
                            <TabsList className="w-full h-12 rounded-lg grid grid-cols-3 p-1">
                                <TabsTrigger value="random" className="rounded-md">Random</TabsTrigger>
                                <TabsTrigger value="passphrase" className="rounded-md">Passphrase</TabsTrigger>
                                <TabsTrigger value="pronounceable" className="rounded-md">Pronounceable</TabsTrigger>
                            </TabsList>

                            <div className="p-6">
                                <TabsContent value="random" className="space-y-6 mt-0">
                                    <div className="space-y-4">
                                        <div className="flex items-center justify-between">
                                            <Label>Length: {config.length}</Label>
                                            <span className="text-xs text-muted-foreground">4 - 128 characters</span>
                                        </div>
                                        <Slider
                                            value={[config.length]}
                                            onValueChange={(v) => updateConfig('length', v[0])}
                                            min={4}
                                            max={128}
                                            step={1}
                                        />
                                    </div>

                                    <div className="grid grid-cols-2 gap-4">
                                        <div className="flex items-center space-x-2 border p-3 rounded-md">
                                            <Checkbox
                                                id="uppercase"
                                                checked={config.useUppercase}
                                                onCheckedChange={(c) => updateConfig('useUppercase', c)}
                                            />
                                            <Label htmlFor="uppercase" className="flex-1 cursor-pointer">A-Z Uppercase</Label>
                                        </div>
                                        <div className="flex items-center space-x-2 border p-3 rounded-md">
                                            <Checkbox
                                                id="lowercase"
                                                checked={config.useLowercase}
                                                onCheckedChange={(c) => updateConfig('useLowercase', c)}
                                            />
                                            <Label htmlFor="lowercase" className="flex-1 cursor-pointer">a-z Lowercase</Label>
                                        </div>
                                        <div className="flex items-center space-x-2 border p-3 rounded-md">
                                            <Checkbox
                                                id="numbers"
                                                checked={config.useNumbers}
                                                onCheckedChange={(c) => updateConfig('useNumbers', c)}
                                            />
                                            <Label htmlFor="numbers" className="flex-1 cursor-pointer">0-9 Numbers</Label>
                                        </div>
                                        <div className="flex items-center space-x-2 border p-3 rounded-md">
                                            <Checkbox
                                                id="symbols"
                                                checked={config.useSymbols}
                                                onCheckedChange={(c) => updateConfig('useSymbols', c)}
                                            />
                                            <Label htmlFor="symbols" className="flex-1 cursor-pointer">!@# Symbols</Label>
                                        </div>
                                        <div className="flex items-center space-x-2 border p-3 rounded-md col-span-2">
                                            <Checkbox
                                                id="ambiguous"
                                                checked={config.excludeAmbiguous}
                                                onCheckedChange={(c) => updateConfig('excludeAmbiguous', c)}
                                            />
                                            <Label htmlFor="ambiguous" className="flex-1 cursor-pointer">Exclude Ambiguous (0 O I l 1)</Label>
                                        </div>
                                    </div>
                                </TabsContent>

                                <TabsContent value="passphrase" className="space-y-6 mt-0">
                                    <div className="space-y-4">
                                        <div className="flex items-center justify-between">
                                            <Label>Word Count: {config.wordCount}</Label>
                                            <span className="text-xs text-muted-foreground">3 - 12 words</span>
                                        </div>
                                        <Slider
                                            value={[config.wordCount]}
                                            onValueChange={(v) => updateConfig('wordCount', v[0])}
                                            min={3}
                                            max={12}
                                            step={1}
                                        />
                                    </div>
                                    <div className="space-y-2">
                                        <Label htmlFor="separator">Separator</Label>
                                        <Input
                                            id="separator"
                                            value={config.separator}
                                            onChange={(e) => updateConfig('separator', e.target.value)}
                                            maxLength={1}
                                            className="font-mono text-center w-20"
                                        />
                                    </div>
                                </TabsContent>

                                <TabsContent value="pronounceable" className="space-y-6 mt-0">
                                    <div className="space-y-4">
                                        <div className="flex items-center justify-between">
                                            <Label>Syllables: {config.syllableCount}</Label>
                                            <span className="text-xs text-muted-foreground">3 - 12 syllables</span>
                                        </div>
                                        <Slider
                                            value={[config.syllableCount]}
                                            onValueChange={(v) => updateConfig('syllableCount', v[0])}
                                            min={3}
                                            max={12}
                                            step={1}
                                        />
                                    </div>
                                </TabsContent>
                            </div>
                        </Tabs>
                    </div>
                </div>

                {/* History Sidebar */}
                <div className="w-[280px] bg-muted/10 border-l flex flex-col">
                    <div className="p-4 border-b bg-muted/20">
                        <h3 className="font-semibold text-sm flex items-center gap-2">
                            <Clock className="h-4 w-4" /> Recent History
                        </h3>
                    </div>
                    <div className="flex-1 overflow-y-auto p-2 space-y-2">
                        {history.length === 0 ? (
                            <div className="text-center py-8 text-muted-foreground text-xs">
                                Copied passwords will appear here.
                            </div>
                        ) : (
                            history.map((pw, i) => (
                                <div key={i} className="group relative bg-card border rounded-md p-2 text-sm font-mono break-all hover:bg-accent hover:border-accent-foreground/50 transition-colors">
                                    <div className="pr-6">{pw}</div>
                                    <Button
                                        variant="ghost"
                                        size="icon"
                                        className="absolute right-1 top-1 h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity"
                                        onClick={() => handleCopy(pw)}
                                    >
                                        <Copy className="h-3 w-3" />
                                    </Button>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
