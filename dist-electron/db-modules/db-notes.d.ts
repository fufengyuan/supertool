import { type NoteRecord, type NoteGroupRecord } from './db-core';
export declare function getAllNoteGroups(): NoteGroupRecord[];
export declare function addNoteGroup(data: {
    id?: string;
    name: string;
    icon?: string;
    sortOrder?: number;
}): NoteGroupRecord;
export declare function updateNoteGroup(id: string, updates: {
    name?: string;
    icon?: string;
    sortOrder?: number;
}): NoteGroupRecord | null;
export declare function deleteNoteGroup(id: string): boolean;
export declare function getAllNotes(query?: string, groupId?: string | null): NoteRecord[];
export declare function getNoteById(id: string): NoteRecord | null;
export declare function addNote(data: {
    id?: string;
    title?: string;
    content?: string;
    tags?: string;
    pinned?: boolean;
    groupId?: string | null;
}): NoteRecord;
export declare function updateNote(id: string, updates: {
    title?: string;
    content?: string;
    tags?: string;
    pinned?: boolean;
    groupId?: string | null;
}): NoteRecord | null;
export declare function deleteNote(id: string): boolean;
