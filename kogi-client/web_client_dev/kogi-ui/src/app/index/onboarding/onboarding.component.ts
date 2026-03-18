import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';

@Component({
  selector: 'app-onboarding',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './onboarding.component.html',
  styleUrls: ['./onboarding.component.css'],
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class OnboardingComponent {
  currentStep = 1;
  router = inject(Router);
  max_steps = 4;

  goToNextStep() {
    if (this.currentStep < this.max_steps) {
      this.currentStep++;
    } else {
      // Onboarding complete, navigate to workspace creation
      this.router.navigate(['/creating-workspace']);
    }
  }
}
