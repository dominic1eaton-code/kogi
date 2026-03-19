import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-itembook-catalogue',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './itembook-catalogue.component.html',
  styleUrl: './itembook-catalogue.component.css',
  host: {
    class: 'block w-full'
  }
})
export class ItembookCatalogueComponent {
  catalogueEntries = [
    { title: 'Research Brief - Community', tag: 'knowledge', status: 'Active' },
    { title: 'Brand System Checklist', tag: 'asset', status: 'Approved' },
    { title: 'Growth Sprint Plan', tag: 'program', status: 'Draft' },
    { title: 'Resource Directory', tag: 'resource', status: 'Active' }
  ];

  tags = ['knowledge', 'asset', 'program', 'resource', 'policy', 'timeline'];
}
