declare const _default: {
    getAccountingCategories: () => Promise<any>;
    addAccountingCategory: (category: {
        id?: string;
        name: string;
        type: string;
        icon?: string;
        sortOrder?: number;
    }) => Promise<any>;
    updateAccountingCategory: (id: string, updates: {
        name?: string;
        type?: string;
        icon?: string;
        sortOrder?: number;
    }) => Promise<any>;
    deleteAccountingCategory: (id: string) => Promise<{
        success: boolean;
        error?: string;
    }>;
    getAccountingRecords: (options?: {
        page?: number;
        pageSize?: number;
        type?: string;
        category?: string;
        status?: string;
        payment_method?: string;
        entity?: string;
        project?: string;
        startDate?: string;
        endDate?: string;
        search?: string;
    }) => Promise<any>;
    addAccountingRecord: (record: {
        id?: string;
        date: string;
        type: string;
        category: string;
        amount: number;
        description?: string;
        status?: string;
        attachmentPath?: string;
        createdBy?: string;
        voucher_number?: string;
        entity?: string;
        project?: string;
        supplier?: string;
        invoice_number?: string;
        tax_amount?: number;
        payment_method?: string;
        approver?: string;
        attachments_json?: string;
    }) => Promise<any>;
    updateAccountingRecord: (id: string, updates: {
        date?: string;
        type?: string;
        category?: string;
        amount?: number;
        description?: string;
        status?: string;
        attachmentPath?: string;
        voucher_number?: string;
        entity?: string;
        project?: string;
        supplier?: string;
        invoice_number?: string;
        tax_amount?: number;
        payment_method?: string;
        approver?: string;
        attachments_json?: string;
    }) => Promise<any>;
    deleteAccountingRecord: (id: string) => Promise<any>;
    getAccountingStats: (options?: {
        startDate?: string;
        endDate?: string;
    }) => Promise<any>;
    exportAccountingCSV: (options?: {
        startDate?: string;
        endDate?: string;
        type?: string;
        category?: string;
        search?: string;
    }) => Promise<any>;
    initDefaultAccountingCategories: () => Promise<any>;
    uploadAccountingReceipt: (fileName: string, base64Data: string) => Promise<any>;
    getAccountingReceiptFile: (filePath: string) => Promise<any>;
    getAccountingTrend: (months?: number) => Promise<any>;
    getBudgets: () => Promise<any>;
    addBudget: (data: {
        category: string;
        amount: number;
        period?: string;
    }) => Promise<any>;
    updateBudget: (id: string, updates: {
        category?: string;
        amount?: number;
        period?: string;
    }) => Promise<any>;
    deleteBudget: (id: string) => Promise<{
        success: boolean;
        error?: string;
    }>;
    checkBudgetAlerts: () => Promise<any>;
    getTemplates: () => Promise<any>;
    addTemplate: (data: {
        name: string;
        type: string;
        category: string;
        amount: number;
        description?: string;
        entity?: string;
        project?: string;
        supplier?: string;
        payment_method?: string;
        tax_rate?: number;
    }) => Promise<any>;
    updateTemplate: (id: string, updates: {
        name?: string;
        type?: string;
        category?: string;
        amount?: number;
        description?: string;
        entity?: string;
        project?: string;
        supplier?: string;
        payment_method?: string;
        tax_rate?: number;
    }) => Promise<any>;
    deleteTemplate: (id: string) => Promise<{
        success: boolean;
        error?: string;
    }>;
    useTemplate: (id: string) => Promise<{
        success: boolean;
        error?: string;
    }>;
};
export default _default;
