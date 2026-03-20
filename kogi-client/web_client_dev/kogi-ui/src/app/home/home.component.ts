import { Component } from '@angular/core';
import { NgFor } from '@angular/common';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [NgFor, RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './home.component.html',
  styleUrl: './home.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class HomeComponent {
  navItems = [
    { label: 'Home', route: '/home', exact: true },
    { label: 'About', route: '/home/about' },
    { label: 'Platform', route: '/home/platform' },
    { label: 'Ecosystem', route: '/home/ecosystem' }
  ];
}
