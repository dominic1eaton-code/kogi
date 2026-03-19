import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-itembook-library',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './itembook-library.component.html',
  styleUrl: './itembook-library.component.css',
  host: {
    class: 'block w-full'
  }
})
export class ItembookLibraryComponent {
  libraryAssets = [
    { name: 'Growth Playbook', type: 'Playbook', status: 'Active' },
    { name: 'Brand Charter Template', type: 'Template', status: 'Approved' },
    { name: 'Resource Intake Workflow', type: 'Workflow', status: 'Active' },
    { name: 'Contribution Agreement', type: 'Policy', status: 'Draft' }
  ];

  templates = ['Project Charter', 'Program Overview', 'Resource Request', 'Governance Review'];
}
