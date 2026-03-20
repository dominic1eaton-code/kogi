import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-home-landing',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './home-landing.component.html',
  styleUrl: './home-landing.component.css'
})
export class HomeLandingComponent {}
