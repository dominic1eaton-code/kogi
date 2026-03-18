import { Component } from '@angular/core';
import { NavigationComponent } from '../navigation/navigation.component';

@Component({
  selector: 'app-hub',
  standalone: true,
  imports: [NavigationComponent],
  templateUrl: './hub.component.html',
  styleUrl: './hub.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class HubComponent {}
