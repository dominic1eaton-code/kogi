import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Router, RouterLink } from '@angular/router';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink],
  templateUrl: './login.component.html',
  host: {
    class: 'block min-h-screen w-full'
  }
})
export class LoginComponent {
  email = 'jordan@studio.io';
  password = '';
  remember = true;
  showPassword = false;

  private router = inject(Router);

  togglePassword() {
    this.showPassword = !this.showPassword;
  }

  login() {
    console.log('Login attempt', {
      email: this.email,
      password: this.password,
      remember: this.remember
    });

    this.router.navigate(['/dashboard']);
  }
}
