declare const _default: {
    getNoteGroups: () => Promise<any>;
    addNoteGroup: (data: {
        id?: string;
        name: string;
        icon?: string;
        sortOrder?: number;
    }) => Promise<any>;
    updateNoteGroup: (id: string, updates: {
        name?: string;
        icon?: string;
        sortOrder?: number;
    }) => Promise<any>;
    deleteNoteGroup: (id: string) => Promise<any>;
    getAllNotes: (query?: string, groupId?: string | null) => Promise<any>;
    getNoteById: (id: string) => Promise<any>;
    addNote: (data: {
        id?: string;
        title?: string;
        content?: string;
        tags?: string;
        pinned?: boolean;
        groupId?: string | null;
    }) => Promise<any>;
    updateNote: (id: string, updates: {
        title?: string;
        content?: string;
        tags?: string;
        pinned?: boolean;
        groupId?: string | null;
    }) => Promise<any>;
    deleteNote: (id: string) => Promise<any>;
};
export default _default;
