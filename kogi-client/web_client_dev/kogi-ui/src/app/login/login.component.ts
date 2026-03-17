// import { Component } from '@angular/core';
// import { CommonModule } from '@angular/common';
// import { FormsModule } from '@angular/forms';

// @Component({
//   selector: 'app-login',
//   standalone: true,
//   imports: [CommonModule, FormsModule],
//   host: { class: 'block w-screen h-screen' },
//   templateUrl: './login.html',
//   styleUrl: './login.css',
//   styles: [`
//     :host {
//       display:block;
//       min-height:100vh
//     }
//     `]
// })
// export class LoginComponent {
//   email = 'jordan@studio.io';
//   password = '••••••••••••';
//   remember = true;
//   showPassword = false;

//   togglePassword() {
//     this.showPassword = !this.showPassword;
//   }

//   login() {
//     console.log('Login attempt', {
//       email: this.email,
//       password: this.password,
//       remember: this.remember
//     });
//   }
// }


import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './login.component.html',
  styleUrls: ['./login.component.css'],
  host: {
    class: 'block w-screen h-screen'
  }
})
export class LoginComponent {

  email = 'jordan@studio.io';
  password = '••••••••••••';
  remember = true;
  showPassword = false;

  togglePassword() {
    this.showPassword = !this.showPassword;
  }

  login() {
    console.log('Login attempt', {
      email: this.email,
      password: this.password,
      remember: this.remember
    });
  }
}

