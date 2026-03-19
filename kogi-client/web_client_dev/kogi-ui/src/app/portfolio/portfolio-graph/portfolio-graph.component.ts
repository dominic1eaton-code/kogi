import { CommonModule } from '@angular/common';
import { Component } from '@angular/core';

@Component({
  selector: 'app-portfolio-graph',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './portfolio-graph.component.html',
  styleUrl: './portfolio-graph.component.css',
  host: {
    class: 'block w-full'
  }
})
export class PortfolioGraphComponent {
  nodes = ['Alpha Platform', 'Community Growth', 'Brand Identity', 'Research Hub', 'Design Assets'];
  edges = [
    { from: 'Alpha Platform', to: 'Community Growth', label: 'contains' },
    { from: 'Community Growth', to: 'Brand Identity', label: 'depends on' },
    { from: 'Brand Identity', to: 'Design Assets', label: 'uses' },
    { from: 'Research Hub', to: 'Brand Identity', label: 'contributes' }
  ];
}
