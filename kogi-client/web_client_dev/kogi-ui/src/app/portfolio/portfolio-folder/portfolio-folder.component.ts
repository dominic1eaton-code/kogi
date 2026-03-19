import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-folder',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-folder.component.html',
  styleUrl: './portfolio-folder.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioFolderComponent {
  folderTree = [
    {
      name: 'Brand Identity System',
      children: ['Guidelines', 'Assets', 'Exports', 'Research']
    },
    {
      name: 'Community Growth Program',
      children: ['Campaigns', 'Content', 'Partners']
    },
    {
      name: 'Portfolio Engine',
      children: ['Specs', 'Roadmap', 'Release Notes']
    }
  ];

  folderMeta = [
    { label: 'Total Folders', value: '28' },
    { label: 'Files', value: '412' },
    { label: 'Last Sync', value: 'Mar 18, 2026' }
  ];
}
