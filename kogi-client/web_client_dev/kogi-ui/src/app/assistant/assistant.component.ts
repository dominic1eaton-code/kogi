import { Component } from '@angular/core';
import { NavigationComponent } from '../navigation/navigation.component';

@Component({
  selector: 'app-assistant',
  standalone: true,
  imports: [NavigationComponent],
  templateUrl: './assistant.component.html',
  styleUrl: './assistant.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class AssistantComponent {}
