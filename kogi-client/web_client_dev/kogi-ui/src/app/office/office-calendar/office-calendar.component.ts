import { Component } from '@angular/core';

@Component({
  selector: 'app-office-calendar',
  standalone: true,
  templateUrl: './office-calendar.component.html',
  styleUrl: './office-calendar.component.css',
  host: {
    class: 'block w-full'
  }
})
export class OfficeCalendarComponent {}
