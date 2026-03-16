import {
    CreditCard,
    FileText,
    KeyRound,
    Lock,
    File,
    FileImage,
    FileAudio,
    FileVideo,
    FileArchive,
    FileCode,
    LayoutGrid,
    Star,
    Trash2,
    FileLock,
} from "lucide-react";

export const getCategoryIcon = (type: string) => {
    switch (type) {
        case 'login': return Lock;
        case 'card': return CreditCard;
        case 'note': return FileText;
        case 'code': return KeyRound;
        case 'file': return FileLock;
        case 'all': return LayoutGrid;
        case 'favorites': return Star;
        case 'trash': return Trash2;
        default: return File;
    }
};

export const getFileIcon = (filename: string) => {
    if (!filename) return File;
    const ext = filename.split('.').pop()?.toLowerCase();

    switch (ext) {
        // Images
        case 'png':
        case 'jpg':
        case 'jpeg':
        case 'gif':
        case 'svg':
        case 'webp':
        case 'bmp':
        case 'ico':
            return FileImage;

        // Audio
        case 'mp3':
        case 'wav':
        case 'ogg':
        case 'm4a':
        case 'flac':
            return FileAudio;

        // Video
        case 'mp4':
        case 'mkv':
        case 'avi':
        case 'mov':
        case 'webm':
            return FileVideo;

        // Archives
        case 'zip':
        case 'rar':
        case '7z':
        case 'tar':
        case 'gz':
        case 'iso':
        case 'kaps': // Our encrypted format
        case 'kept': // Our backup format
            return FileArchive;

        // Code
        case 'js':
        case 'jsx':
        case 'ts':
        case 'tsx':
        case 'html':
        case 'css':
        case 'json':
        case 'py':
        case 'rs':
        case 'java':
        case 'c':
        case 'cpp':
        case 'h':
        case 'go':
        case 'sql':
        case 'xml':
        case 'yaml':
        case 'yml':
            return FileCode;

        // Text / Documents
        case 'txt':
        case 'md':
        case 'pdf':
        case 'doc':
        case 'docx':
        case 'xls':
        case 'xlsx':
        case 'ppt':
        case 'pptx':
        case 'odt':
            return FileText;

        default:
            return File;
    }
};
