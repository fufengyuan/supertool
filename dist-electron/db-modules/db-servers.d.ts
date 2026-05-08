import { type Server, type ServerGroup } from './db-core';
declare function rowToServer(row: any): Server;
declare function getAllServers(): Server[];
declare function getServerById(serverId: string): Server | null;
declare function addServer(server: Server): Server | null;
declare function updateServer(server: Server): Server | null;
declare function deleteServer(serverId: string): {
    success: boolean;
};
declare function rowToServerGroup(row: any): ServerGroup;
declare function getAllServerGroups(): ServerGroup[];
declare function addServerGroup(group: {
    id?: string;
    name: string;
    description?: string;
    parentId?: string | null;
    color?: string;
}): ServerGroup | undefined;
declare function updateServerGroup(groupId: string, updates: {
    name?: string;
    description?: string;
    parentId?: string | null;
    color?: string;
}): ServerGroup | undefined;
declare function deleteServerGroup(groupId: string): {
    success: boolean;
};
declare const _default: {
    rowToServer: typeof rowToServer;
    getAllServers: typeof getAllServers;
    getServerById: typeof getServerById;
    addServer: typeof addServer;
    updateServer: typeof updateServer;
    deleteServer: typeof deleteServer;
    rowToServerGroup: typeof rowToServerGroup;
    getAllServerGroups: typeof getAllServerGroups;
    addServerGroup: typeof addServerGroup;
    updateServerGroup: typeof updateServerGroup;
    deleteServerGroup: typeof deleteServerGroup;
};
export = _default;
