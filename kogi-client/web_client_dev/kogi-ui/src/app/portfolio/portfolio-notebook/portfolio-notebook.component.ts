import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-notebook',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-notebook.component.html',
  styleUrl: './portfolio-notebook.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioNotebookComponent {
  notes = [
    { title: 'Growth Hypotheses', body: 'Test retention loop improvements via new onboarding flow.' },
    { title: 'Resource Constraints', body: 'Design team bandwidth limited until May.' },
    { title: 'Portfolio Health', body: 'Governance compliance trending above 85%.' }
  ];

  topics = ['strategy', 'research', 'governance', 'funding', 'operations'];
}
