import { WORDLIST } from './wordlist';

export interface GeneratorConfig {
    length: number;
    useUppercase: boolean;
    useLowercase: boolean;
    useNumbers: boolean;
    useSymbols: boolean;
    excludeAmbiguous: boolean;

    // Passphrase
    wordCount: number;
    separator: string;

    // Pronounceable
    syllableCount: number;
}

const UPPERCASE = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
const LOWERCASE = 'abcdefghijklmnopqrstuvwxyz';
const NUMBERS = '0123456789';
const SYMBOLS = '!@#$%^&*()_+-=[]{}|;:,.<>?';
const AMBIGUOUS = '0O1Il5S';

const CONSONANTS = 'bcdfghjklmnpqrstvwxyz';
const VOWELS = 'aeiou';

export function calculateEntropy(password: string): number {
    if (!password) return 0;

    let poolSize = 0;
    if (/[a-z]/.test(password)) poolSize += 26;
    if (/[A-Z]/.test(password)) poolSize += 26;
    if (/[0-9]/.test(password)) poolSize += 10;
    if (/[^a-zA-Z0-9]/.test(password)) poolSize += 32;

    if (poolSize === 0) return 0;

    const entropy = Math.log2(Math.pow(poolSize, password.length));
    return Math.round(entropy);
}

export function generateRandomPassword(config: GeneratorConfig): string {
    let charset = '';
    if (config.useLowercase) charset += LOWERCASE;
    if (config.useUppercase) charset += UPPERCASE;
    if (config.useNumbers) charset += NUMBERS;
    if (config.useSymbols) charset += SYMBOLS;

    if (config.excludeAmbiguous) {
        charset = charset.split('').filter(c => !AMBIGUOUS.includes(c)).join('');
    }

    if (charset.length === 0) return '';

    const array = new Uint32Array(config.length);
    window.crypto.getRandomValues(array);

    let password = '';
    for (let i = 0; i < config.length; i++) {
        password += charset[array[i] % charset.length];
    }
    return password;
}

export function generatePassphrase(config: GeneratorConfig): string {
    const words: string[] = [];
    const array = new Uint32Array(config.wordCount);
    window.crypto.getRandomValues(array);

    for (let i = 0; i < config.wordCount; i++) {
        words.push(WORDLIST[array[i] % WORDLIST.length]);
    }

    return words.join(config.separator);
}

export function generatePronounceable(config: GeneratorConfig): string {
    let password = '';
    const array = new Uint32Array(config.syllableCount * 2);
    window.crypto.getRandomValues(array);

    for (let i = 0; i < config.syllableCount; i++) {
        const c = CONSONANTS[array[i * 2] % CONSONANTS.length];
        const v = VOWELS[array[i * 2 + 1] % VOWELS.length];
        password += (i === 0 ? c.toUpperCase() : c) + v;
    }

    return password;
}
