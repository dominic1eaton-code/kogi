// web/src/app/features/stories/stories-board.component.ts
import { Component, OnInit } from '@angular/core';
import { CdkDragDrop, moveItemInArray, transferArrayItem } from '@angular/cdk/drag-drop';
import { MatDialog } from '@angular/material/dialog';
import { MatSnackBar } from '@angular/material/snack-bar';
import { PortfolioService } from '../../core/services/portfolio.service';
import { Story, StoryStatus, STORY_STATUS_COLORS } from '../../core/models/models';

interface KanbanColumn {
  id: StoryStatus;
  label: string;
  stories: Story[];
  color: string;
}

@Component({
  selector: 'kogi-stories-board',
  template: `
    <div class="board-header">
      <h2>Story Board</h2>
      <div class="board-actions">
        <mat-form-field appearance="outline" class="search-field">
          <mat-icon matPrefix>search</mat-icon>
          <input matInput placeholder="Filter stories..." [(ngModel)]="filterText"
                 (input)="onFilter()">
        </mat-form-field>
        <button mat-raised-button color="primary" (click)="openCreateDialog()">
          <mat-icon>add</mat-icon> New Story
        </button>
      </div>
    </div>

    <div class="kanban-board" cdkDropListGroup>
      <div *ngFor="let col of columns" class="kanban-col">
        <div class="col-header" [style.border-top-color]="col.color">
          <span class="col-title">{{ col.label }}</span>
          <mat-chip class="col-count" [style.background]="col.color">{{ col.stories.length }}</mat-chip>
        </div>

        <div class="col-body"
             cdkDropList
             [id]="col.id"
             [cdkDropListData]="col.stories"
             [cdkDropListConnectedTo]="connectedTo"
             (cdkDropListDropped)="onDrop($event, col.id)">

          <mat-card *ngFor="let story of col.stories"
                    class="story-card"
                    cdkDrag
                    (click)="openStoryDetail(story)">
            <div class="story-type-badge" [style.background]="typeColor(story.type)">
              {{ story.type }}
            </div>
            <div class="story-title">{{ story.title }}</div>
            <div class="story-footer">
              <mat-chip class="priority-chip" [style.background]="priorityColor(story.priority)">
                {{ story.priority }}
              </mat-chip>
              <span *ngIf="story.points" class="story-points">{{ story.points }} pts</span>
              <mat-icon *ngIf="story.assignee_id" class="assigned-icon"
                        matTooltip="Assigned">person</mat-icon>
            </div>
          </mat-card>

          <div *ngIf="col.stories.length === 0" class="empty-col">
            Drop stories here
          </div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .board-header { display: flex; justify-content: space-between; align-items: center;
                    margin-bottom: 20px; color: #F1F5F9; }
    .board-header h2 { font-size: 20px; font-weight: 700; margin: 0; }
    .board-actions { display: flex; gap: 12px; align-items: center; }
    .search-field  { width: 220px; }
    .kanban-board  { display: flex; gap: 16px; overflow-x: auto; padding-bottom: 16px; min-height: 600px; }
    .kanban-col    { flex: 0 0 260px; display: flex; flex-direction: column; }
    .col-header    { display: flex; justify-content: space-between; align-items: center;
                     padding: 10px 12px; background: #1E293B; border-radius: 8px 8px 0 0;
                     border-top: 3px solid; }
    .col-title     { font-weight: 600; font-size: 13px; color: #F1F5F9; }
    .col-count     { height: 20px; min-height: 20px; font-size: 11px; color: white !important; }
    .col-body      { flex: 1; background: #0F172A; border: 1px solid #334155;
                     border-top: none; border-radius: 0 0 8px 8px;
                     padding: 8px; min-height: 400px; }
    .story-card    { margin-bottom: 8px; background: #1E293B !important;
                     border: 1px solid #334155; cursor: grab; padding: 10px; }
    .story-card:hover { border-color: #3B82F6; }
    .story-type-badge { font-size: 10px; font-weight: 600; padding: 2px 6px;
                        border-radius: 3px; display: inline-block; margin-bottom: 6px;
                        color: white; text-transform: uppercase; }
    .story-title   { font-size: 13px; color: #F1F5F9; margin-bottom: 8px; line-height: 1.4; }
    .story-footer  { display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }
    .priority-chip { font-size: 10px; height: 18px; min-height: 18px; color: white !important; }
    .story-points  { font-size: 11px; color: #64748B; margin-left: auto; }
    .assigned-icon { font-size: 16px; color: #60A5FA; width: 16px; height: 16px; }
    .empty-col     { text-align: center; color: #334155; font-size: 12px; padding: 24px 0;
                     border: 2px dashed #334155; border-radius: 6px; margin: 4px; }
    .cdk-drag-placeholder { opacity: 0.4; background: #334155 !important; }
    .cdk-drag-animating    { transition: transform 250ms ease; }
  `]
})
export class StoriesBoardComponent implements OnInit {
  filterText = '';
  allStories: Story[] = [];

  columns: KanbanColumn[] = [
    { id: 'backlog',     label: 'Backlog',      stories: [], color: '#475569' },
    { id: 'ready',       label: 'Ready',        stories: [], color: '#2563EB' },
    { id: 'in_progress', label: 'In Progress',  stories: [], color: '#D97706' },
    { id: 'in_review',   label: 'In Review',    stories: [], color: '#7C3AED' },
    { id: 'done',        label: 'Done',         stories: [], color: '#059669' },
  ];

  get connectedTo(): string[] {
    return this.columns.map(c => c.id);
  }

  constructor(
    private portfolioService: PortfolioService,
    private snackBar: MatSnackBar,
    private dialog: MatDialog
  ) {}

  ngOnInit(): void {
    // TODO: load from real WBS
    this.seedDemoStories();
  }

  private seedDemoStories(): void {
    const demo: Story[] = [
      { id: '1', wbs_id: 'wbs-1', type: 'feature', status: 'in_progress',
        title: 'User can create a portfolio', priority: 'high', points: 5,
        description: '', created_at: '', updated_at: '' },
      { id: '2', wbs_id: 'wbs-1', type: 'bug', status: 'backlog',
        title: 'Login fails on mobile Safari', priority: 'critical', points: 2,
        description: '', created_at: '', updated_at: '' },
      { id: '3', wbs_id: 'wbs-1', type: 'milestone', status: 'ready',
        title: 'v0.1.0 alpha release', priority: 'high',
        description: '', created_at: '', updated_at: '' },
      { id: '4', wbs_id: 'wbs-1', type: 'research', status: 'done',
        title: 'Evaluate Zig kernel approach', priority: 'medium', points: 3,
        description: '', created_at: '', updated_at: '' },
    ];
    this.allStories = demo;
    this.distributeStories(demo);
  }

  private distributeStories(stories: Story[]): void {
    this.columns.forEach(c => c.stories = []);
    stories.forEach(s => {
      const col = this.columns.find(c => c.id === s.status);
      if (col) col.stories.push(s);
    });
  }

  onDrop(event: CdkDragDrop<Story[]>, targetStatus: StoryStatus): void {
    if (event.previousContainer === event.container) {
      moveItemInArray(event.container.data, event.previousIndex, event.currentIndex);
    } else {
      transferArrayItem(event.previousContainer.data, event.container.data,
        event.previousIndex, event.currentIndex);
      const movedStory = event.container.data[event.currentIndex];
      movedStory.status = targetStatus;
      this.snackBar.open(`Story moved to ${targetStatus.replace('_', ' ')}`, '', { duration: 2000 });
      // TODO: persist status change via portfolioService.updateStoryStatus()
    }
  }

  onFilter(): void {
    if (!this.filterText.trim()) {
      this.distributeStories(this.allStories);
      return;
    }
    const q = this.filterText.toLowerCase();
    this.distributeStories(this.allStories.filter(s =>
      s.title.toLowerCase().includes(q) || s.type.includes(q)
    ));
  }

  openCreateDialog(): void {
    this.snackBar.open('Create story dialog — coming soon', '', { duration: 2000 });
  }

  openStoryDetail(story: Story): void {
    this.snackBar.open(`Story: ${story.title}`, '', { duration: 1500 });
  }

  typeColor(type: string): string {
    const map: Record<string, string> = {
      feature: '#1D4ED8', bug: '#DC2626', milestone: '#7C3AED',
      research: '#0891B2', design: '#D97706', idea: '#059669', spike: '#EA580C',
    };
    return map[type] ?? '#475569';
  }

  priorityColor(priority: string): string {
    const map: Record<string, string> = {
      critical: '#DC2626', high: '#EA580C', medium: '#CA8A04', low: '#475569',
    };
    return map[priority] ?? '#475569';
  }
}
