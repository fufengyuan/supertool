interface Project {
    id: string;
    name: string;
    description?: string;
    color?: string;
    createdAt: string;
    updatedAt: string;
    archived?: boolean;
}
interface ProjectStat {
    projectId: string;
    daysActive: number;
}
interface Task {
    text: string;
    [key: string]: unknown;
}
interface GitCommit {
    projectName: string;
    message: string;
    author: string;
    date: string;
}
interface ReportData {
    startDate: string;
    endDate: string;
    completedTasks: Task[];
    projects: Project[];
    projectStats: ProjectStat[];
    weeklyWork: Record<string, Task[]>;
    gitCommits: GitCommit[];
    nextWeekPlan: Record<string, Task[]>;
}
declare function createReportDocx(reportData: ReportData): Promise<Buffer>;
declare const _default: {
    createReportDocx: typeof createReportDocx;
};
export = _default;
