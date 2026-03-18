import { Component } from '@angular/core';

@Component({
  selector: 'app-dashboard-overview',
  standalone: true,
  templateUrl: './dashboard-overview.component.html',
  styleUrl: './dashboard-overview.component.css',
  host: {
    class: 'block w-full'
  }
})
export class DashboardOverviewComponent {}
