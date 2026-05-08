"use strict";
const docx_1 = require("docx");
function formatDate(date) {
    if (!date)
        return '';
    const d = new Date(date);
    return d.toLocaleDateString('zh-CN');
}
function getProjectName(projectId, projects) {
    if (projectId === 'unassigned')
        return '未分类任务';
    const project = projects.find(p => p.id === projectId);
    return project ? project.name : '未知项目';
}
// 创建表格行
function createTableRow(cells) {
    return new docx_1.TableRow({
        children: cells.map(text => new docx_1.TableCell({
            children: [new docx_1.Paragraph({
                    children: [new docx_1.TextRun(text)],
                    spacing: { before: 100, after: 100 }
                })],
            width: { size: 5000, type: docx_1.WidthType.PERCENTAGE }
        })),
    });
}
// 创建带样式的单元格
function createStyledCell(text, isHeader = false) {
    return new docx_1.TableCell({
        children: [new docx_1.Paragraph({
                children: [new docx_1.TextRun({
                        text: text,
                        bold: isHeader,
                    })],
                spacing: { before: 100, after: 100 }
            })],
        shading: isHeader ? {
            fill: 'DDDDDD',
        } : undefined,
    });
}
// 创建表格
function createTable(headerCells, dataRows) {
    const headerRow = new docx_1.TableRow({
        children: headerCells.map(cellText => createStyledCell(cellText, true)),
    });
    const dataRowsObjects = dataRows.map(rowData => new docx_1.TableRow({
        children: rowData.map(cellText => createStyledCell(cellText)),
    }));
    return new docx_1.Table({
        rows: [headerRow, ...dataRowsObjects],
        width: { size: 100, type: docx_1.WidthType.PERCENTAGE },
    });
}
async function createReportDocx(reportData) {
    // 构建文档内容
    const doc = new docx_1.Document({
        sections: [{
                properties: {},
                children: [
                    // 标题
                    new docx_1.Paragraph({
                        text: '周报',
                        heading: docx_1.HeadingLevel.HEADING_1,
                        spacing: { after: 300 }
                    }),
                    // 摘要信息
                    new docx_1.Paragraph({
                        text: `时间范围: ${formatDate(reportData.startDate)} 至 ${formatDate(reportData.endDate)}`,
                        spacing: { after: 200 }
                    }),
                    new docx_1.Paragraph({
                        text: `完成任务数: ${reportData.completedTasks.length}`,
                        spacing: { after: 200 }
                    }),
                    new docx_1.Paragraph({
                        text: `涉及项目数: ${reportData.projects.length}`,
                        spacing: { after: 400 }
                    }),
                    // 项目统计表
                    new docx_1.Paragraph({
                        text: '项目统计表',
                        heading: docx_1.HeadingLevel.HEADING_2,
                        spacing: { before: 400, after: 200 }
                    }),
                    createTable(['项目', '耗时天数'], reportData.projectStats.map(stat => [
                        getProjectName(stat.projectId, reportData.projects),
                        stat.daysActive.toString()
                    ])),
                    // 本周工作表
                    new docx_1.Paragraph({
                        text: '本周工作总结',
                        heading: docx_1.HeadingLevel.HEADING_2,
                        spacing: { before: 400, after: 200 }
                    }),
                    ...Object.entries(reportData.weeklyWork).map(([projectId, tasks]) => [
                        new docx_1.Paragraph({
                            text: getProjectName(projectId, reportData.projects),
                            heading: docx_1.HeadingLevel.HEADING_3,
                            spacing: { before: 300, after: 100 }
                        }),
                        ...tasks.map(task => new docx_1.Paragraph({
                            text: `• ${task.text}`,
                            spacing: { before: 100, after: 100 }
                        }))
                    ]).flat(),
                    // Git提交记录表
                    ...(reportData.gitCommits.length > 0 ? [
                        new docx_1.Paragraph({
                            text: 'Git提交记录',
                            heading: docx_1.HeadingLevel.HEADING_2,
                            spacing: { before: 400, after: 200 }
                        }),
                        createTable(['项目', '提交', '作者', '日期'], reportData.gitCommits.map(commit => [
                            commit.projectName,
                            commit.message,
                            commit.author,
                            formatDate(commit.date)
                        ]))
                    ] : []),
                    // 下周计划表
                    new docx_1.Paragraph({
                        text: '下周工作计划',
                        heading: docx_1.HeadingLevel.HEADING_2,
                        spacing: { before: 400, after: 200 }
                    }),
                    ...Object.entries(reportData.nextWeekPlan).map(([projectId, tasks]) => [
                        new docx_1.Paragraph({
                            text: getProjectName(projectId, reportData.projects),
                            heading: docx_1.HeadingLevel.HEADING_3,
                            spacing: { before: 300, after: 100 }
                        }),
                        ...tasks.map(task => new docx_1.Paragraph({
                            text: `• ${task.text}`,
                            spacing: { before: 100, after: 100 }
                        }))
                    ]).flat(),
                ],
            }],
    });
    return await docx_1.Packer.toBuffer(doc);
}
module.exports = {
    createReportDocx
};
//# sourceMappingURL=export-word.js.map