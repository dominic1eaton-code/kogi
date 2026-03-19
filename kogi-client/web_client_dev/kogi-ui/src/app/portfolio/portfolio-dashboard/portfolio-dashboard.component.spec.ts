import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterTestingModule } from '@angular/router/testing';

import { PortfolioDashboardComponent } from './portfolio-dashboard.component';

describe('PortfolioDashboardComponent', () => {
  let component: PortfolioDashboardComponent;
  let fixture: ComponentFixture<PortfolioDashboardComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioDashboardComponent, RouterTestingModule]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioDashboardComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
